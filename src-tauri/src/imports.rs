use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUrlInfo {
    pub source_type: String,
    pub normalized_url: String,
    pub is_supported: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplateDraft {
    pub id: String,
    pub title: String,
    pub category: String,
    pub source_repo: String,
    pub source_url: String,
    pub source_license: Option<String>,
    pub author: Option<String>,
    pub model_hint: String,
    pub language: String,
    pub prompt_original: String,
    pub prompt_zh: Option<String>,
    pub prompt_en: Option<String>,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: Option<String>,
    pub tags: Vec<String>,
    pub preview_image_urls: Vec<String>,
    pub imported_at: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub source: ImportUrlInfo,
    pub items: Vec<PromptTemplateDraft>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub warnings: Vec<String>,
}

pub fn classify_import_url(url: &str) -> ImportUrlInfo {
    let trimmed = url.trim().to_string();
    let source_type = if trimmed.contains("raw.githubusercontent.com") {
        "github_raw"
    } else if trimmed.contains("github.com") && trimmed.contains("/blob/") {
        "github_blob"
    } else if trimmed.contains("github.com") && trimmed.contains("/tree/") {
        "github_tree"
    } else if trimmed.contains("github.com") {
        "github_repo"
    } else {
        "unknown"
    };

    ImportUrlInfo {
        is_supported: source_type != "unknown",
        normalized_url: trimmed,
        source_type: source_type.to_string(),
    }
}

pub fn preview_import_url(url: &str) -> Result<ImportPreview, String> {
    let source = classify_import_url(url);
    if !source.is_supported {
        return Err("Unsupported import URL. Paste a GitHub repository, blob, or raw URL.".to_string());
    }

    let documents = fetch_import_documents(url)?;
    let mut warnings = Vec::new();
    let mut items = Vec::new();

    for document in documents {
        match parse_prompt_document(url, &document.url, &document.content) {
            Ok(mut parsed) => items.append(&mut parsed),
            Err(err) => warnings.push(format!("{}: {err}", document.url)),
        }
    }

    if items.is_empty() && warnings.is_empty() {
        warnings.push("No prompt-like Markdown or JSON entries were found.".to_string());
    }

    Ok(ImportPreview { source, items, warnings })
}

pub fn import_prompt_library(workspace_root: &std::path::Path, url: &str) -> Result<ImportResult, String> {
    let preview = preview_import_url(url)?;
    let workspace = crate::workspace::ensure_workspace(workspace_root)?;
    db::bootstrap(std::path::Path::new(&workspace.database_path))?;
    let imported_count = db::insert_prompt_templates(std::path::Path::new(&workspace.database_path), &preview.items)?;

    Ok(ImportResult {
        imported_count,
        skipped_count: preview.items.len().saturating_sub(imported_count),
        warnings: preview.warnings,
    })
}

pub fn parse_prompt_document(
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Result<Vec<PromptTemplateDraft>, String> {
    if source_url.to_ascii_lowercase().ends_with(".json") || content.trim_start().starts_with('[') || content.trim_start().starts_with('{') {
        return parse_json_prompts(source_repo, source_url, content);
    }

    Ok(parse_markdown_prompts(source_repo, source_url, content))
}

fn parse_markdown_prompts(source_repo: &str, source_url: &str, content: &str) -> Vec<PromptTemplateDraft> {
    let mut current_category = String::new();
    let mut current_title = String::new();
    let mut current_prompt: Option<String> = None;
    let mut current_negative: Option<String> = None;
    let mut current_aspect: Option<String> = None;
    let mut items = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(title) = line.strip_prefix("## ") {
            flush_markdown_item(
                source_repo,
                source_url,
                &current_category,
                &current_title,
                &mut current_prompt,
                &mut current_negative,
                &mut current_aspect,
                &mut items,
            );
            current_category = clean_heading(title);
            current_title.clear();
            continue;
        }

        if let Some(title) = line.strip_prefix("### ") {
            flush_markdown_item(
                source_repo,
                source_url,
                &current_category,
                &current_title,
                &mut current_prompt,
                &mut current_negative,
                &mut current_aspect,
                &mut items,
            );
            current_title = clean_heading(title);
            continue;
        }

        if let Some(value) = extract_labeled_value(line, &["Prompt:", "Prompt："]) {
            current_prompt = Some(value);
            continue;
        }

        if let Some(value) = extract_labeled_value(
            line,
            &["Negative Prompts:", "Negative Prompt:", "Negative:", "负面提示词：", "负面提示词:"],
        ) {
            current_negative = Some(value);
            continue;
        }

        if let Some(value) = extract_labeled_value(line, &["Aspect Ratio:", "Aspect ratio:", "Ratio:", "比例：", "比例:"]) {
            current_aspect = Some(value);
            continue;
        }
    }

    flush_markdown_item(
        source_repo,
        source_url,
        &current_category,
        &current_title,
        &mut current_prompt,
        &mut current_negative,
        &mut current_aspect,
        &mut items,
    );

    items
}

fn flush_markdown_item(
    source_repo: &str,
    source_url: &str,
    category: &str,
    title: &str,
    prompt: &mut Option<String>,
    negative_prompt: &mut Option<String>,
    aspect_ratio: &mut Option<String>,
    items: &mut Vec<PromptTemplateDraft>,
) {
    if let Some(prompt_original) = prompt.take() {
        if prompt_original.trim().is_empty() {
            return;
        }
        items.push(build_prompt_draft(
            source_repo,
            source_url,
            if title.is_empty() { "Imported Prompt" } else { title },
            category,
            prompt_original,
            negative_prompt.take(),
            aspect_ratio.take(),
            Vec::new(),
        ));
    }
}

fn parse_json_prompts(source_repo: &str, source_url: &str, content: &str) -> Result<Vec<PromptTemplateDraft>, String> {
    let value: Value = serde_json::from_str(content).map_err(|err| format!("Invalid JSON: {err}"))?;
    let mut items = Vec::new();
    collect_json_prompts(source_repo, source_url, &value, "", &mut items);
    Ok(items)
}

fn collect_json_prompts(
    source_repo: &str,
    source_url: &str,
    value: &Value,
    category_hint: &str,
    items: &mut Vec<PromptTemplateDraft>,
) {
    match value {
        Value::Array(values) => {
            for item in values {
                collect_json_prompts(source_repo, source_url, item, category_hint, items);
            }
        }
        Value::Object(map) => {
            let prompt = string_field(map, &["prompt", "promptOriginal", "prompt_original", "text", "content"]);
            if let Some(prompt_original) = prompt {
                let title = string_field(map, &["title", "name"]).unwrap_or_else(|| "Imported Prompt".to_string());
                let category = string_field(map, &["category", "type"]).unwrap_or_else(|| category_hint.to_string());
                let negative_prompt = string_field(map, &["negativePrompt", "negative_prompt", "negative", "negativePrompts"]);
                let aspect_ratio = string_field(map, &["aspectRatio", "aspect_ratio", "ratio"]);
                let tags = tags_field(map.get("tags"));
                items.push(build_prompt_draft(
                    source_repo,
                    source_url,
                    &title,
                    &category,
                    prompt_original,
                    negative_prompt,
                    aspect_ratio,
                    tags,
                ));
                return;
            }

            for (key, child) in map {
                collect_json_prompts(source_repo, source_url, child, key, items);
            }
        }
        _ => {}
    }
}

fn build_prompt_draft(
    source_repo: &str,
    source_url: &str,
    title: &str,
    category: &str,
    prompt_original: String,
    negative_prompt: Option<String>,
    aspect_ratio: Option<String>,
    tags: Vec<String>,
) -> PromptTemplateDraft {
    let content_hash = hash_template(source_url, &prompt_original);
    PromptTemplateDraft {
        id: content_hash.clone(),
        title: title.trim().to_string(),
        category: category.trim().to_string(),
        source_repo: source_repo.to_string(),
        source_url: source_url.to_string(),
        source_license: None,
        author: None,
        model_hint: infer_model_hint(source_repo),
        language: infer_language(&prompt_original),
        prompt_original,
        prompt_zh: None,
        prompt_en: None,
        negative_prompt,
        aspect_ratio,
        tags,
        preview_image_urls: Vec::new(),
        imported_at: current_timestamp(),
        content_hash,
    }
}

#[derive(Debug)]
struct ImportDocument {
    url: String,
    content: String,
}

fn fetch_import_documents(url: &str) -> Result<Vec<ImportDocument>, String> {
    let info = classify_import_url(url);
    match info.source_type.as_str() {
        "github_raw" => Ok(vec![ImportDocument {
            url: url.to_string(),
            content: http_get_text(url)?,
        }]),
        "github_blob" => {
            let raw_url = github_blob_to_raw(url)?;
            Ok(vec![ImportDocument {
                url: raw_url.clone(),
                content: http_get_text(&raw_url)?,
            }])
        }
        "github_repo" | "github_tree" => fetch_github_repo_documents(url),
        _ => Err("Unsupported import URL.".to_string()),
    }
}

fn fetch_github_repo_documents(url: &str) -> Result<Vec<ImportDocument>, String> {
    let repo = parse_github_repo(url)?;
    let metadata_url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let metadata: Value = http_get_json(&metadata_url)?;
    let branch = metadata
        .get("default_branch")
        .and_then(Value::as_str)
        .unwrap_or("main");

    let tree_url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
        repo.owner, repo.name, branch
    );
    let tree: Value = http_get_json(&tree_url)?;
    let mut documents = Vec::new();

    if let Some(entries) = tree.get("tree").and_then(Value::as_array) {
        for entry in entries {
            let Some(path) = entry.get("path").and_then(Value::as_str) else {
                continue;
            };
            let lower = path.to_ascii_lowercase();
            if !is_supported_prompt_file(&lower) {
                continue;
            }

            let raw_url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                repo.owner, repo.name, branch, path
            );
            if let Ok(content) = http_get_text(&raw_url) {
                documents.push(ImportDocument { url: raw_url, content });
            }
            if documents.len() >= 20 {
                break;
            }
        }
    }

    if documents.is_empty() {
        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/README.md",
            repo.owner, repo.name, branch
        );
        documents.push(ImportDocument {
            url: raw_url.clone(),
            content: http_get_text(&raw_url)?,
        });
    }

    Ok(documents)
}

#[derive(Debug)]
struct GitHubRepo {
    owner: String,
    name: String,
}

fn parse_github_repo(url: &str) -> Result<GitHubRepo, String> {
    let without_query = url.split('?').next().unwrap_or(url);
    let marker = "github.com/";
    let Some(index) = without_query.find(marker) else {
        return Err("Not a GitHub URL.".to_string());
    };
    let rest = &without_query[index + marker.len()..];
    let mut parts = rest.split('/').filter(|part| !part.is_empty());
    let owner = parts.next().ok_or_else(|| "GitHub owner missing.".to_string())?;
    let name = parts.next().ok_or_else(|| "GitHub repository missing.".to_string())?;
    Ok(GitHubRepo {
        owner: owner.to_string(),
        name: name.trim_end_matches(".git").to_string(),
    })
}

fn github_blob_to_raw(url: &str) -> Result<String, String> {
    let repo = parse_github_repo(url)?;
    let marker = "/blob/";
    let Some(index) = url.find(marker) else {
        return Err("GitHub blob URL is missing /blob/.".to_string());
    };
    let rest = &url[index + marker.len()..];
    let mut parts = rest.splitn(2, '/');
    let branch = parts.next().ok_or_else(|| "GitHub blob branch missing.".to_string())?;
    let path = parts.next().ok_or_else(|| "GitHub blob path missing.".to_string())?;
    Ok(format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        repo.owner, repo.name, branch, path
    ))
}

fn http_get_text(url: &str) -> Result<String, String> {
    let response = ureq::get(url)
        .header("User-Agent", "PromptWeave")
        .call()
        .map_err(|err| format!("Failed to fetch {url}: {err}"))?;
    response
        .into_body()
        .read_to_string()
        .map_err(|err| format!("Failed to read {url}: {err}"))
}

fn http_get_json(url: &str) -> Result<Value, String> {
    let text = http_get_text(url)?;
    serde_json::from_str(&text).map_err(|err| format!("Invalid JSON from {url}: {err}"))
}

fn is_supported_prompt_file(path: &str) -> bool {
    if !(path.ends_with(".md") || path.ends_with(".json")) {
        return false;
    }
    path == "readme.md"
        || path.contains("prompt")
        || path.contains("gpt-image")
        || path.contains("image")
        || path.contains("example")
        || path.contains("case")
}

fn extract_labeled_value(line: &str, labels: &[&str]) -> Option<String> {
    labels.iter().find_map(|label| {
        line.strip_prefix(label)
            .map(|value| value.trim().trim_matches('`').trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn clean_heading(value: &str) -> String {
    value.trim().trim_matches('#').trim().to_string()
}

fn string_field(map: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        map.get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn tags_field(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        Some(Value::String(value)) => value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

fn hash_template(source_url: &str, prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_url.as_bytes());
    hasher.update(b"\n");
    hasher.update(prompt.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn infer_model_hint(source_repo: &str) -> String {
    let lower = source_repo.to_ascii_lowercase();
    if lower.contains("gpt-image-2") {
        "gpt-image-2".to_string()
    } else if lower.contains("gpt-image") {
        "gpt-image".to_string()
    } else {
        "generic".to_string()
    }
}

fn infer_language(prompt: &str) -> String {
    if prompt.chars().any(|char| ('\u{4e00}'..='\u{9fff}').contains(&char)) {
        "zh".to_string()
    } else {
        "en".to_string()
    }
}

fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_github_repo_url() {
        let info = classify_import_url("https://github.com/EvoLinkAI/awesome-gpt-image-2-prompts");
        assert_eq!(info.source_type, "github_repo");
        assert!(info.is_supported);
    }

    #[test]
    fn extracts_prompt_templates_from_markdown_prompt_blocks() {
        let markdown = r#"
# Awesome GPT Image

## Portrait

### Cinematic Girl

Prompt: A young girl wearing a red cloak standing on a snowy mountain peak.

Negative Prompts: blurry, watermark

Aspect Ratio: 16:9
"#;

        let items = parse_prompt_document(
            "https://github.com/example/repo",
            "https://raw.githubusercontent.com/example/repo/main/README.md",
            markdown,
        )
        .expect("markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Cinematic Girl");
        assert_eq!(items[0].category, "Portrait");
        assert_eq!(
            items[0].prompt_original,
            "A young girl wearing a red cloak standing on a snowy mountain peak."
        );
        assert_eq!(items[0].negative_prompt.as_deref(), Some("blurry, watermark"));
        assert_eq!(items[0].aspect_ratio.as_deref(), Some("16:9"));
    }

    #[test]
    fn extracts_prompt_templates_from_json_array() {
        let json = r#"
[
  {
    "title": "Product Poster",
    "category": "Poster",
    "prompt": "A premium perfume bottle on reflective glass.",
    "negativePrompt": "low quality",
    "aspectRatio": "1:1",
    "tags": ["product", "poster"]
  }
]
"#;

        let items = parse_prompt_document(
            "https://github.com/example/repo",
            "https://raw.githubusercontent.com/example/repo/main/prompts.json",
            json,
        )
        .expect("json should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Product Poster");
        assert_eq!(items[0].category, "Poster");
        assert_eq!(items[0].prompt_original, "A premium perfume bottle on reflective glass.");
        assert_eq!(items[0].tags, vec!["product".to_string(), "poster".to_string()]);
    }
}
