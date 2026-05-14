use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::db;

const DEFAULT_GPT_IMAGE_2_OWNER: &str = "EvoLinkAI";
const DEFAULT_GPT_IMAGE_2_REPO: &str = "awesome-gpt-image-2-API-and-Prompts";
const DEFAULT_GPT_IMAGE_2_CASES_URL: &str =
    "https://github.com/EvoLinkAI/awesome-gpt-image-2-API-and-Prompts/tree/main/cases";
const FREESTYLEFLY_GALLERY_URL: &str =
    "https://github.com/freestylefly/awesome-gpt-image-2/blob/main/docs/gallery.md";
const FREESTYLEFLY_PART_1_RAW_URL: &str =
    "https://raw.githubusercontent.com/freestylefly/awesome-gpt-image-2/main/docs/gallery-part-1.md";
const FREESTYLEFLY_PART_2_RAW_URL: &str =
    "https://raw.githubusercontent.com/freestylefly/awesome-gpt-image-2/main/docs/gallery-part-2.md";
const YOUMIND_README_ZH_URL: &str =
    "https://github.com/YouMind-OpenLab/awesome-gpt-image-2/blob/main/README_zh.md";
const PROMPT_LABELS: &[&str] = &[
    "Prompt:",
    "Prompt：",
    "提示词:",
    "提示词：",
    "正向提示词:",
    "正向提示词：",
    "中文提示词:",
    "中文提示词：",
    "简体中文:",
    "简体中文：",
];
const METADATA_LABELS: &[&str] = &[
    "Title:",
    "Title：",
    "标题:",
    "标题：",
    "Category:",
    "Category：",
    "分类:",
    "分类：",
    "Language:",
    "Language：",
    "语言:",
    "语言：",
    "Model:",
    "Model：",
    "模型:",
    "模型：",
    "Author:",
    "Author：",
    "作者:",
    "作者：",
    "Aspect Ratio:",
    "Aspect Ratio：",
    "Aspect ratio:",
    "Aspect ratio：",
    "Ratio:",
    "Ratio：",
    "比例:",
    "比例：",
    "Negative Prompts:",
    "Negative Prompts：",
    "Negative Prompt:",
    "Negative Prompt：",
    "Negative:",
    "Negative：",
    "负面提示词:",
    "负面提示词：",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportAdapterKind {
    EvoLinkCases,
    FreestyleflyGallery,
    YouMindReadmeZh,
    Generic,
}

#[derive(Debug, Clone, Copy, Default)]
struct GitHubRepoFetchOptions {
    prefer_simplified_case_paths: bool,
}

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
    pub source_id: String,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub warnings: Vec<String>,
}

pub fn classify_import_url(url: &str) -> ImportUrlInfo {
    let normalized_url = normalize_import_url(url);
    let source_type = if normalized_url.contains("raw.githubusercontent.com") {
        "github_raw"
    } else if normalized_url.contains("github.com") && normalized_url.contains("/blob/") {
        "github_blob"
    } else if normalized_url.contains("github.com") && normalized_url.contains("/tree/") {
        "github_tree"
    } else if normalized_url.contains("github.com") {
        "github_repo"
    } else {
        "unknown"
    };

    ImportUrlInfo {
        is_supported: source_type != "unknown",
        normalized_url,
        source_type: source_type.to_string(),
    }
}

pub fn preview_import_url(url: &str) -> Result<ImportPreview, String> {
    let source = classify_import_url(url);
    if !source.is_supported {
        return Err(
            "Unsupported import URL. Paste a GitHub repository, blob, or raw URL.".to_string(),
        );
    }

    let adapter = resolve_import_adapter(&source.normalized_url);
    let documents = fetch_import_documents_for_adapter(adapter, &source.normalized_url)?;
    let mut warnings = Vec::new();
    let mut items = Vec::new();

    for document in documents {
        match parse_prompt_document_for_adapter(
            adapter,
            &source.normalized_url,
            &document.url,
            &document.content,
        ) {
            Ok(mut parsed) => items.append(&mut parsed),
            Err(err) => warnings.push(format!("{}: {err}", document.url)),
        }
    }

    items = post_process_import_items(adapter, items);

    if items.is_empty() && warnings.is_empty() {
        warnings.push("No prompt-like Markdown or JSON entries were found.".to_string());
    }

    Ok(ImportPreview {
        source,
        items,
        warnings,
    })
}

fn normalize_import_url(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(normalized) = normalize_builtin_import_url(trimmed) {
        return normalized;
    }
    trimmed.to_string()
}

fn normalize_builtin_import_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let without_query = trimmed
        .split(['?', '#'])
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches('/');
    let lower = without_query.to_ascii_lowercase();
    if lower
        == "https://raw.githubusercontent.com/freestylefly/awesome-gpt-image-2/main/docs/gallery.md"
    {
        return Some(FREESTYLEFLY_GALLERY_URL.to_string());
    }
    if lower
        == "https://raw.githubusercontent.com/youmind-openlab/awesome-gpt-image-2/main/readme_zh.md"
    {
        return Some(YOUMIND_README_ZH_URL.to_string());
    }

    let Ok(repo) = parse_github_repo(without_query) else {
        return None;
    };

    if is_default_gpt_image_2_repo(&repo)
        && (is_github_repo_root_url(without_query)
            || is_default_gpt_image_2_cases_url(without_query))
    {
        return Some(DEFAULT_GPT_IMAGE_2_CASES_URL.to_string());
    }

    if repo.owner.eq_ignore_ascii_case("freestylefly")
        && repo.name.eq_ignore_ascii_case("awesome-gpt-image-2")
        && (lower.ends_with("/blob/main/docs/gallery.md")
            || lower.ends_with("/main/docs/gallery.md"))
    {
        return Some(FREESTYLEFLY_GALLERY_URL.to_string());
    }

    if repo.owner.eq_ignore_ascii_case("YouMind-OpenLab")
        && repo.name.eq_ignore_ascii_case("awesome-gpt-image-2")
        && (lower.ends_with("/blob/main/readme_zh.md") || lower.ends_with("/main/readme_zh.md"))
    {
        return Some(YOUMIND_README_ZH_URL.to_string());
    }

    None
}

fn resolve_import_adapter(url: &str) -> ImportAdapterKind {
    let normalized = normalize_import_url(url);
    if is_default_gpt_image_2_cases_url(&normalized) {
        ImportAdapterKind::EvoLinkCases
    } else if normalized == FREESTYLEFLY_GALLERY_URL {
        ImportAdapterKind::FreestyleflyGallery
    } else if normalized == YOUMIND_README_ZH_URL {
        ImportAdapterKind::YouMindReadmeZh
    } else {
        ImportAdapterKind::Generic
    }
}

fn is_github_repo_root_url(url: &str) -> bool {
    let without_query = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim_end_matches('/');
    let Some(index) = without_query.find("github.com/") else {
        return false;
    };
    let rest = &without_query[index + "github.com/".len()..];
    let mut parts = rest.split('/').filter(|part| !part.is_empty());
    parts.next().is_some() && parts.next().is_some() && parts.next().is_none()
}

pub fn import_prompt_library(
    workspace_root: &std::path::Path,
    url: &str,
) -> Result<ImportResult, String> {
    let preview = preview_import_url(url)?;
    let workspace = crate::workspace::ensure_workspace(workspace_root)?;
    db::bootstrap(std::path::Path::new(&workspace.database_path))?;
    let database_path = std::path::Path::new(&workspace.database_path);
    let source_id = source_id(&preview.source.normalized_url);
    let synced_at = current_timestamp();
    db::upsert_prompt_library_source(
        database_path,
        &db::PromptLibrarySourceDraft {
            id: source_id.clone(),
            name: source_name(&preview.source.normalized_url),
            url: preview.source.normalized_url.clone(),
            source_type: preview.source.source_type.clone(),
            created_at: synced_at.clone(),
        },
    )?;
    let imported_count = db::insert_prompt_templates(database_path, &preview.items)?;
    let skipped_count = preview.items.len().saturating_sub(imported_count);
    db::record_prompt_library_source_success(
        database_path,
        &source_id,
        imported_count,
        skipped_count,
        &synced_at,
    )?;

    Ok(ImportResult {
        source_id,
        imported_count,
        skipped_count,
        warnings: preview.warnings,
    })
}

pub fn list_prompt_library_sources(
    workspace_root: &std::path::Path,
) -> Result<Vec<db::PromptLibrarySourceRecord>, String> {
    let workspace = crate::workspace::ensure_workspace(workspace_root)?;
    db::bootstrap(std::path::Path::new(&workspace.database_path))?;
    db::list_prompt_library_sources(std::path::Path::new(&workspace.database_path))
}

pub fn sync_prompt_library_source(
    workspace_root: &std::path::Path,
    source_id: &str,
) -> Result<ImportResult, String> {
    let workspace = crate::workspace::ensure_workspace(workspace_root)?;
    let database_path = std::path::Path::new(&workspace.database_path);
    db::bootstrap(database_path)?;
    let source = db::get_prompt_library_source(database_path, source_id)?;
    match import_prompt_library(workspace_root, &source.url) {
        Ok(result) => Ok(result),
        Err(err) => {
            let _ = db::record_prompt_library_source_error(
                database_path,
                source_id,
                &err,
                &current_timestamp(),
            );
            Err(err)
        }
    }
}

pub fn parse_prompt_document(
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Result<Vec<PromptTemplateDraft>, String> {
    let adapter = match resolve_import_adapter(source_repo) {
        ImportAdapterKind::Generic => resolve_import_adapter(source_url),
        adapter => adapter,
    };
    parse_prompt_document_for_adapter(adapter, source_repo, source_url, content)
}

fn parse_prompt_document_for_adapter(
    adapter: ImportAdapterKind,
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Result<Vec<PromptTemplateDraft>, String> {
    match adapter {
        ImportAdapterKind::FreestyleflyGallery => Ok(parse_freestylefly_gallery_prompts(
            source_repo,
            source_url,
            content,
        )),
        ImportAdapterKind::YouMindReadmeZh => Ok(parse_youmind_readme_zh_prompts(
            source_repo,
            source_url,
            content,
        )),
        ImportAdapterKind::EvoLinkCases | ImportAdapterKind::Generic => {
            parse_prompt_document_generic(source_repo, source_url, content)
        }
    }
}

fn parse_prompt_document_generic(
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Result<Vec<PromptTemplateDraft>, String> {
    if source_url.to_ascii_lowercase().ends_with(".json")
        || content.trim_start().starts_with('[')
        || content.trim_start().starts_with('{')
    {
        return parse_json_prompts(source_repo, source_url, content);
    }

    Ok(parse_markdown_prompts(source_repo, source_url, content))
}

fn post_process_import_items(
    adapter: ImportAdapterKind,
    items: Vec<PromptTemplateDraft>,
) -> Vec<PromptTemplateDraft> {
    match adapter {
        ImportAdapterKind::EvoLinkCases => prefer_simplified_chinese_items(items),
        ImportAdapterKind::FreestyleflyGallery
        | ImportAdapterKind::YouMindReadmeZh
        | ImportAdapterKind::Generic => items,
    }
}

fn parse_freestylefly_gallery_prompts(
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Vec<PromptTemplateDraft> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut index = 0;
    let mut current_title: Option<String> = None;
    let mut current_images: Vec<String> = Vec::new();
    let mut items = Vec::new();

    while index < lines.len() {
        let line = lines[index].trim();
        if let Some(title) = freestylefly_case_title(line) {
            current_title = Some(title);
            current_images.clear();
            index += 1;
            continue;
        }

        if current_title.is_some() {
            if let Some(image_url) = extract_image_url(line) {
                current_images.push(image_url);
                index += 1;
                continue;
            }

            if is_freestylefly_prompt_label(line) {
                if let Some((prompt, next_index)) = extract_next_fenced_block(&lines, index + 1) {
                    let prompt = extract_preferred_chinese_section(&prompt);
                    let prompt = clean_prompt_text(&prompt);
                    if !prompt.is_empty() {
                        let title = current_title.as_deref().unwrap_or("Imported Prompt");
                        let mut draft = build_prompt_draft(
                            source_repo,
                            source_url,
                            title,
                            "freestylefly gallery",
                            prompt,
                            None,
                            None,
                            Vec::new(),
                        );
                        draft.preview_image_urls = std::mem::take(&mut current_images);
                        items.push(draft);
                    }
                    current_title = None;
                    index = next_index;
                    continue;
                }
            }
        }

        index += 1;
    }

    items
}

fn parse_youmind_readme_zh_prompts(
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Vec<PromptTemplateDraft> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut index = 0;
    let mut current_title: Option<String> = None;
    let mut items = Vec::new();

    while index < lines.len() {
        let line = lines[index].trim();
        if let Some(title) = youmind_case_title(line) {
            current_title = Some(title);
            index += 1;
            continue;
        }

        if current_title.is_some() && is_youmind_prompt_section(line) {
            if let Some((prompt, next_index)) = extract_next_fenced_block(&lines, index + 1) {
                let prompt = prompt.trim().to_string();
                if !prompt.is_empty() {
                    let mut preview_image_urls = Vec::new();
                    let mut scan_index = next_index;
                    while scan_index < lines.len() {
                        let scan_line = lines[scan_index].trim();
                        if youmind_case_title(scan_line).is_some() {
                            break;
                        }
                        if let Some(image_url) = extract_image_url(scan_line) {
                            preview_image_urls.push(image_url);
                        }
                        scan_index += 1;
                    }

                    let title = current_title.as_deref().unwrap_or("Imported Prompt");
                    let mut draft = build_prompt_draft(
                        source_repo,
                        source_url,
                        title,
                        "YouMind README_zh",
                        prompt,
                        None,
                        None,
                        Vec::new(),
                    );
                    draft.preview_image_urls = preview_image_urls;
                    items.push(draft);
                }
                current_title = None;
                index = next_index;
                continue;
            }
        }

        index += 1;
    }

    items
}

fn parse_markdown_prompts(
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Vec<PromptTemplateDraft> {
    let mut current_category = String::new();
    let mut current_title = String::new();
    let mut current_prompt: Option<String> = None;
    let mut current_negative: Option<String> = None;
    let mut current_aspect: Option<String> = None;
    let mut current_author: Option<String> = None;
    let mut current_images: Vec<String> = Vec::new();
    let mut in_prompt_fence = false;
    let mut pending_prompt_fence = false;
    let mut fence_buffer: Vec<String> = Vec::new();
    let mut items = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if in_prompt_fence {
            if line.starts_with("```") {
                in_prompt_fence = false;
                let prompt = fence_buffer.join("\n").trim().to_string();
                fence_buffer.clear();
                if !prompt.is_empty() {
                    current_prompt = Some(prompt);
                }
            } else {
                fence_buffer.push(line.to_string());
            }
            continue;
        }

        if line.starts_with("```") {
            let fence_lang = line.trim_start_matches('`').trim().to_ascii_lowercase();
            if fence_lang.contains("prompt") || pending_prompt_fence {
                in_prompt_fence = true;
                pending_prompt_fence = false;
                fence_buffer.clear();
            }
            continue;
        }

        if let Some(title) = line.strip_prefix("## ") {
            flush_markdown_item(
                source_repo,
                source_url,
                &current_category,
                &current_title,
                current_author.as_deref(),
                &mut current_prompt,
                &mut current_negative,
                &mut current_aspect,
                &mut current_images,
                &mut items,
            );
            current_category = clean_heading(title);
            current_title.clear();
            current_author = None;
            pending_prompt_fence = false;
            continue;
        }

        if let Some(title) = line.strip_prefix("### ") {
            flush_markdown_item(
                source_repo,
                source_url,
                &current_category,
                &current_title,
                current_author.as_deref(),
                &mut current_prompt,
                &mut current_negative,
                &mut current_aspect,
                &mut current_images,
                &mut items,
            );
            let (title, author) = split_title_author(&clean_heading(title));
            current_title = title;
            current_author = author;
            pending_prompt_fence = false;
            continue;
        }

        if let Some(image_url) = extract_markdown_image_url(line) {
            current_images.push(image_url);
            continue;
        }

        if let Some(value) = extract_labeled_value(line, PROMPT_LABELS) {
            current_prompt = Some(value);
            continue;
        }
        if starts_labeled_line(line, PROMPT_LABELS) {
            pending_prompt_fence = true;
            continue;
        }

        if let Some(value) = extract_labeled_value(
            line,
            &[
                "Negative Prompts:",
                "Negative Prompt:",
                "Negative:",
                "负面提示词：",
                "负面提示词:",
            ],
        ) {
            current_negative = Some(value);
            continue;
        }

        if let Some(value) = extract_labeled_value(
            line,
            &[
                "Aspect Ratio:",
                "Aspect ratio:",
                "Ratio:",
                "比例：",
                "比例:",
            ],
        ) {
            current_aspect = Some(value);
            continue;
        }
    }

    flush_markdown_item(
        source_repo,
        source_url,
        &current_category,
        &current_title,
        current_author.as_deref(),
        &mut current_prompt,
        &mut current_negative,
        &mut current_aspect,
        &mut current_images,
        &mut items,
    );

    items
}

fn flush_markdown_item(
    source_repo: &str,
    source_url: &str,
    category: &str,
    title: &str,
    author: Option<&str>,
    prompt: &mut Option<String>,
    negative_prompt: &mut Option<String>,
    aspect_ratio: &mut Option<String>,
    preview_image_urls: &mut Vec<String>,
    items: &mut Vec<PromptTemplateDraft>,
) {
    if let Some(prompt_original) = prompt.take() {
        let prompt_original = clean_prompt_text(&prompt_original);
        if prompt_original.is_empty() {
            return;
        }
        let mut draft = build_prompt_draft(
            source_repo,
            source_url,
            if title.is_empty() {
                "Imported Prompt"
            } else {
                title
            },
            category,
            prompt_original,
            negative_prompt.take(),
            aspect_ratio.take(),
            Vec::new(),
        );
        draft.author = author.map(ToOwned::to_owned);
        draft.preview_image_urls = std::mem::take(preview_image_urls);
        items.push(draft);
    }
}

fn parse_json_prompts(
    source_repo: &str,
    source_url: &str,
    content: &str,
) -> Result<Vec<PromptTemplateDraft>, String> {
    let value: Value =
        serde_json::from_str(content).map_err(|err| format!("Invalid JSON: {err}"))?;
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
            let prompt = string_field(
                map,
                &[
                    "prompt",
                    "promptOriginal",
                    "prompt_original",
                    "text",
                    "content",
                ],
            );
            if let Some(prompt_original) = prompt {
                let prompt_original = clean_prompt_text(&prompt_original);
                if prompt_original.is_empty() {
                    return;
                }
                let title = string_field(map, &["title", "name"])
                    .unwrap_or_else(|| "Imported Prompt".to_string());
                let category = string_field(map, &["category", "type"])
                    .unwrap_or_else(|| category_hint.to_string());
                let negative_prompt = string_field(
                    map,
                    &[
                        "negativePrompt",
                        "negative_prompt",
                        "negative",
                        "negativePrompts",
                    ],
                );
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

fn fetch_import_documents_for_adapter(
    adapter: ImportAdapterKind,
    url: &str,
) -> Result<Vec<ImportDocument>, String> {
    match adapter {
        ImportAdapterKind::FreestyleflyGallery => fetch_freestylefly_gallery_documents(url),
        ImportAdapterKind::YouMindReadmeZh => fetch_github_blob_or_raw_document(url),
        ImportAdapterKind::EvoLinkCases => fetch_github_repo_documents(
            url,
            GitHubRepoFetchOptions {
                prefer_simplified_case_paths: true,
            },
        ),
        ImportAdapterKind::Generic => {
            let info = classify_import_url(url);
            match info.source_type.as_str() {
                "github_raw" => Ok(vec![ImportDocument {
                    url: url.to_string(),
                    content: http_get_text(url)?,
                }]),
                "github_blob" => fetch_github_blob_or_raw_document(url),
                "github_repo" | "github_tree" => {
                    fetch_github_repo_documents(url, GitHubRepoFetchOptions::default())
                }
                _ => Err("Unsupported import URL.".to_string()),
            }
        }
    }
}

fn fetch_github_blob_or_raw_document(url: &str) -> Result<Vec<ImportDocument>, String> {
    if url.contains("raw.githubusercontent.com") {
        return Ok(vec![ImportDocument {
            url: url.to_string(),
            content: http_get_text(url)?,
        }]);
    }

    let repo = parse_github_repo(url)?;
    let ref_candidates = fetch_github_ref_candidates(&repo).unwrap_or_default();
    let raw_url = if ref_candidates.is_empty() {
        github_blob_to_raw(url)?
    } else {
        github_blob_to_raw_with_refs(url, &ref_candidates)?
    };
    Ok(vec![ImportDocument {
        url: raw_url.clone(),
        content: http_get_text(&raw_url)?,
    }])
}

fn fetch_freestylefly_gallery_documents(url: &str) -> Result<Vec<ImportDocument>, String> {
    let index_documents = fetch_github_blob_or_raw_document(url)?;
    let index_content = index_documents
        .first()
        .map(|document| document.content.as_str())
        .unwrap_or_default();
    let mut part_urls = extract_freestylefly_gallery_part_links(index_content);
    if part_urls.is_empty() {
        part_urls = vec![
            FREESTYLEFLY_PART_1_RAW_URL.to_string(),
            FREESTYLEFLY_PART_2_RAW_URL.to_string(),
        ];
    }

    let mut documents = Vec::new();
    for raw_url in part_urls {
        if let Ok(content) = http_get_text(&raw_url) {
            documents.push(ImportDocument {
                url: raw_url,
                content,
            });
        }
    }

    if documents.is_empty() {
        return Err("No freestylefly gallery part documents were found.".to_string());
    }
    Ok(documents)
}

fn extract_freestylefly_gallery_part_links(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for line in content.lines() {
        if let Some(raw_url) = freestylefly_part_raw_url(line) {
            if !urls.iter().any(|url| url == &raw_url) {
                urls.push(raw_url);
            }
        }
    }
    urls
}

fn freestylefly_part_raw_url(value: &str) -> Option<String> {
    if value.contains("gallery-part-1.md") {
        Some(FREESTYLEFLY_PART_1_RAW_URL.to_string())
    } else if value.contains("gallery-part-2.md") {
        Some(FREESTYLEFLY_PART_2_RAW_URL.to_string())
    } else {
        None
    }
}

fn fetch_github_repo_documents(
    url: &str,
    options: GitHubRepoFetchOptions,
) -> Result<Vec<ImportDocument>, String> {
    let repo = parse_github_repo(url)?;
    let metadata_url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let metadata: Value = http_get_json(&metadata_url)?;
    let default_branch = metadata
        .get("default_branch")
        .and_then(Value::as_str)
        .unwrap_or("main");
    let mut ref_candidates = fetch_github_branch_names(&repo).unwrap_or_default();
    if !ref_candidates.iter().any(|branch| branch == default_branch) {
        ref_candidates.push(default_branch.to_string());
    }
    let selection = parse_github_tree_selection_with_refs(url, default_branch, &ref_candidates)?;
    let branch = selection.branch.as_deref().unwrap_or(default_branch);

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
            if let Some(prefix) = &selection.path_prefix {
                if path != prefix && !path.starts_with(&format!("{prefix}/")) {
                    continue;
                }
            }
            if !is_supported_prompt_file(&lower) {
                continue;
            }
            if options.prefer_simplified_case_paths && is_clearly_non_simplified_case_path(path) {
                continue;
            }

            let raw_url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                repo.owner, repo.name, branch, path
            );
            if let Ok(content) = http_get_text(&raw_url) {
                documents.push(ImportDocument {
                    url: raw_url,
                    content,
                });
            }
            if documents.len() >= 20 {
                break;
            }
        }
    }

    if documents.is_empty() {
        let readme_path = selection
            .path_prefix
            .as_ref()
            .map(|prefix| format!("{prefix}/README.md"))
            .unwrap_or_else(|| "README.md".to_string());
        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            repo.owner, repo.name, branch, readme_path
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

#[derive(Debug, Default)]
struct GitHubTreeSelection {
    branch: Option<String>,
    path_prefix: Option<String>,
}

fn parse_github_repo(url: &str) -> Result<GitHubRepo, String> {
    let without_query = url.split(['?', '#']).next().unwrap_or(url);
    let marker = "github.com/";
    let Some(index) = without_query.find(marker) else {
        return Err("Not a GitHub URL.".to_string());
    };
    let rest = &without_query[index + marker.len()..];
    let mut parts = rest.split('/').filter(|part| !part.is_empty());
    let owner = parts
        .next()
        .ok_or_else(|| "GitHub owner missing.".to_string())?;
    let name = parts
        .next()
        .ok_or_else(|| "GitHub repository missing.".to_string())?;
    Ok(GitHubRepo {
        owner: owner.to_string(),
        name: name.trim_end_matches(".git").to_string(),
    })
}

fn parse_github_tree_selection(
    url: &str,
    default_branch: &str,
) -> Result<GitHubTreeSelection, String> {
    parse_github_tree_selection_with_refs(url, default_branch, &[default_branch.to_string()])
}

fn parse_github_tree_selection_with_refs(
    url: &str,
    default_branch: &str,
    ref_candidates: &[String],
) -> Result<GitHubTreeSelection, String> {
    let Some((_, tree_rest)) = url.split_once("/tree/") else {
        return Ok(GitHubTreeSelection::default());
    };
    let rest = clean_github_ref_rest(tree_rest);
    if rest.is_empty() {
        return Ok(GitHubTreeSelection::default());
    }

    let mut candidates = ref_candidates.to_vec();
    if !candidates.iter().any(|branch| branch == default_branch) {
        candidates.push(default_branch.to_string());
    }

    let (branch, path_prefix) = split_github_ref_path(&rest, &candidates)
        .ok_or_else(|| "GitHub tree branch missing.".to_string())?;
    Ok(GitHubTreeSelection {
        branch: Some(branch),
        path_prefix: path_prefix.and_then(|path| clean_tree_path(&path)),
    })
}

fn clean_github_ref_rest(rest: &str) -> String {
    rest.split(['?', '#'])
        .next()
        .unwrap_or(rest)
        .trim_matches('/')
        .to_string()
}

fn split_github_ref_path(
    rest: &str,
    ref_candidates: &[String],
) -> Option<(String, Option<String>)> {
    let rest = rest.trim_matches('/');
    if rest.is_empty() {
        return None;
    }

    if let Some(branch) = ref_candidates
        .iter()
        .map(|branch| branch.trim_matches('/'))
        .filter(|branch| !branch.is_empty())
        .filter(|branch| rest == *branch || rest.starts_with(&format!("{branch}/")))
        .max_by_key(|branch| branch.len())
    {
        let path = rest
            .strip_prefix(branch)
            .unwrap_or_default()
            .trim_start_matches('/');
        return Some((
            branch.to_string(),
            (!path.is_empty()).then(|| path.to_string()),
        ));
    }

    let mut parts = rest.splitn(2, '/');
    let branch = parts.next()?.to_string();
    let path = parts.next().map(ToOwned::to_owned);
    Some((branch, path))
}

fn clean_tree_path(path: &str) -> Option<String> {
    let normalized = path.trim_matches('/');
    (!normalized.is_empty()).then(|| normalized.to_string())
}

fn github_blob_to_raw(url: &str) -> Result<String, String> {
    github_blob_to_raw_with_refs(url, &[])
}

fn github_blob_to_raw_with_refs(url: &str, ref_candidates: &[String]) -> Result<String, String> {
    let repo = parse_github_repo(url)?;
    let marker = "/blob/";
    let Some(index) = url.find(marker) else {
        return Err("GitHub blob URL is missing /blob/.".to_string());
    };
    let rest = clean_github_ref_rest(&url[index + marker.len()..]);
    let (branch, path) = split_github_ref_path(&rest, ref_candidates)
        .ok_or_else(|| "GitHub blob branch missing.".to_string())?;
    let path = path.ok_or_else(|| "GitHub blob path missing.".to_string())?;
    Ok(format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        repo.owner, repo.name, branch, path
    ))
}

fn fetch_github_ref_candidates(repo: &GitHubRepo) -> Result<Vec<String>, String> {
    let metadata_url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let metadata: Value = http_get_json(&metadata_url)?;
    let mut candidates = fetch_github_branch_names(repo).unwrap_or_default();
    if let Some(default_branch) = metadata.get("default_branch").and_then(Value::as_str) {
        if !candidates.iter().any(|branch| branch == default_branch) {
            candidates.push(default_branch.to_string());
        }
    }
    Ok(candidates)
}

fn fetch_github_branch_names(repo: &GitHubRepo) -> Result<Vec<String>, String> {
    let branches_url = format!(
        "https://api.github.com/repos/{}/{}/branches?per_page=100",
        repo.owner, repo.name
    );
    let branches: Value = http_get_json(&branches_url)?;
    let names = branches
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("name").and_then(Value::as_str))
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(names)
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

fn freestylefly_case_title(line: &str) -> Option<String> {
    let title = line.strip_prefix("### ")?.trim();
    let rest = title.strip_prefix('例')?.trim_start();
    title_after_separator(rest)
}

fn youmind_case_title(line: &str) -> Option<String> {
    let title = line.strip_prefix("### ")?.trim();
    let rest = title.strip_prefix("No.")?.trim_start();
    title_after_separator(rest)
}

fn title_after_separator(value: &str) -> Option<String> {
    let ascii = value.find(':');
    let full_width = value.find('：');
    let index = match (ascii, full_width) {
        (Some(left), Some(right)) => left.min(right),
        (Some(index), None) | (None, Some(index)) => index,
        (None, None) => return None,
    };
    let separator_len = value[index..].chars().next()?.len_utf8();
    let title = value[index + separator_len..].trim();
    (!title.is_empty()).then(|| title.to_string())
}

fn is_freestylefly_prompt_label(line: &str) -> bool {
    let stripped = strip_wrapping_markdown(line);
    starts_labeled_line(&stripped, PROMPT_LABELS)
}

fn is_youmind_prompt_section(line: &str) -> bool {
    let heading = line.trim_start_matches('#').trim();
    heading.contains("提示词")
}

fn extract_next_fenced_block(lines: &[&str], start_index: usize) -> Option<(String, usize)> {
    let mut index = start_index;
    while index < lines.len() && !lines[index].trim().starts_with("```") {
        index += 1;
    }
    if index >= lines.len() {
        return None;
    }

    index += 1;
    let mut buffer = Vec::new();
    while index < lines.len() {
        let line = lines[index].trim();
        if line.starts_with("```") {
            return Some((buffer.join("\n"), index + 1));
        }
        buffer.push(lines[index].to_string());
        index += 1;
    }
    None
}

fn extract_preferred_chinese_section(prompt: &str) -> String {
    let lines = prompt.lines().collect::<Vec<_>>();
    let Some(start) = lines
        .iter()
        .position(|line| line.trim().eq_ignore_ascii_case("[中文]"))
    else {
        return prompt.trim().to_string();
    };

    let mut selected = Vec::new();
    for line in lines.iter().skip(start + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            break;
        }
        selected.push(*line);
    }

    selected.join("\n").trim().to_string()
}

fn extract_image_url(line: &str) -> Option<String> {
    extract_markdown_image_url(line).or_else(|| extract_html_image_url(line))
}

fn extract_html_image_url(line: &str) -> Option<String> {
    if !line.contains("<img") {
        return None;
    }
    let src_index = line.find("src=\"")? + 5;
    let end = line[src_index..].find('"')? + src_index;
    let url = line[src_index..end].trim();
    (!url.is_empty()).then(|| url.to_string())
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

fn is_default_gpt_image_2_repo(repo: &GitHubRepo) -> bool {
    repo.owner.eq_ignore_ascii_case(DEFAULT_GPT_IMAGE_2_OWNER)
        && repo.name.eq_ignore_ascii_case(DEFAULT_GPT_IMAGE_2_REPO)
}

fn is_default_gpt_image_2_cases_url(url: &str) -> bool {
    let Ok(repo) = parse_github_repo(url) else {
        return false;
    };
    if !is_default_gpt_image_2_repo(&repo) {
        return false;
    }
    let lower = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim_end_matches('/')
        .to_ascii_lowercase();
    lower.contains("/tree/") && (lower.ends_with("/cases") || lower.contains("/cases/"))
}

fn is_clearly_non_simplified_case_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.contains("zh-cn")
        || lower.contains("zh_cn")
        || lower.contains("zh-hans")
        || lower.contains("zh_hans")
        || lower.contains("simplified")
        || path.contains("简体")
        || path.contains("中文")
    {
        return false;
    }

    let segments = lower.split(['/', '\\', '.', '_', '-']).collect::<Vec<_>>();
    segments.iter().any(|segment| {
        matches!(
            *segment,
            "en" | "english"
                | "tw"
                | "hant"
                | "traditional"
                | "ja"
                | "jp"
                | "ko"
                | "kr"
                | "fr"
                | "de"
                | "es"
                | "ru"
        )
    }) || path.contains("繁體")
        || path.contains("繁体")
}

fn prefer_simplified_chinese_items(items: Vec<PromptTemplateDraft>) -> Vec<PromptTemplateDraft> {
    let simplified_items = items
        .iter()
        .filter(|item| is_simplified_chinese_prompt(&item.prompt_original))
        .cloned()
        .collect::<Vec<_>>();
    if simplified_items.is_empty() {
        items
    } else {
        simplified_items
    }
}

fn clean_prompt_text(raw: &str) -> String {
    let mut lines = Vec::new();
    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("```") || extract_markdown_image_url(line).is_some()
        {
            continue;
        }

        let mut line = strip_markdown_line_prefixes(line).to_string();
        if starts_labeled_line(&line, METADATA_LABELS) {
            continue;
        }
        if let Some(value) = extract_labeled_value(&line, PROMPT_LABELS) {
            line = value;
        }
        line = strip_markdown_line_prefixes(&line).to_string();
        let cleaned = strip_wrapping_markdown(&line);
        if !cleaned.is_empty() {
            lines.push(cleaned);
        }
    }

    lines.join("\n").trim().to_string()
}

fn strip_markdown_line_prefixes(mut line: &str) -> &str {
    loop {
        let trimmed = line.trim_start();
        if trimmed == line && !has_removable_prefix(trimmed) {
            return trimmed.trim();
        }
        line = trimmed;
        if let Some(rest) = line.strip_prefix('>') {
            line = rest;
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            line = rest.trim_start_matches('#');
            continue;
        }
        if let Some(rest) = line
            .strip_prefix("- ")
            .or_else(|| line.strip_prefix("* "))
            .or_else(|| line.strip_prefix("+ "))
        {
            line = rest;
            continue;
        }
        if let Some(rest) = strip_ordered_prefix(line) {
            line = rest;
            continue;
        }
        return line.trim();
    }
}

fn has_removable_prefix(line: &str) -> bool {
    line.starts_with('>')
        || line.starts_with('#')
        || line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("+ ")
        || strip_ordered_prefix(line).is_some()
}

fn strip_ordered_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if let Some(rest) = strip_parenthesized_number(trimmed, '(', ')') {
        return Some(rest);
    }
    if let Some(rest) = strip_parenthesized_number(trimmed, '（', '）') {
        return Some(rest);
    }

    let digit_end = trimmed
        .char_indices()
        .take_while(|(_, char)| char.is_ascii_digit())
        .map(|(index, char)| index + char.len_utf8())
        .last()?;
    let rest = &trimmed[digit_end..];
    rest.strip_prefix('.')
        .or_else(|| rest.strip_prefix('、'))
        .or_else(|| rest.strip_prefix(')'))
        .map(str::trim_start)
}

fn strip_parenthesized_number(line: &str, open: char, close: char) -> Option<&str> {
    let rest = line.strip_prefix(open)?;
    let digit_end = rest
        .char_indices()
        .take_while(|(_, char)| char.is_ascii_digit())
        .map(|(index, char)| index + char.len_utf8())
        .last()?;
    rest[digit_end..].strip_prefix(close).map(str::trim_start)
}

fn strip_wrapping_markdown(line: &str) -> String {
    let mut value = line.trim();
    loop {
        let next = value
            .strip_prefix("**")
            .and_then(|rest| rest.strip_suffix("**"))
            .or_else(|| {
                value
                    .strip_prefix("__")
                    .and_then(|rest| rest.strip_suffix("__"))
            })
            .or_else(|| {
                value
                    .strip_prefix('`')
                    .and_then(|rest| rest.strip_suffix('`'))
            });
        let Some(stripped) = next else {
            break;
        };
        value = stripped.trim();
    }
    value.to_string()
}

fn starts_labeled_line(line: &str, labels: &[&str]) -> bool {
    labels
        .iter()
        .any(|label| line.trim_start().starts_with(label))
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

fn split_title_author(title: &str) -> (String, Option<String>) {
    let Some(start) = title.rfind("(by ") else {
        return (title.to_string(), None);
    };
    if !title.ends_with(')') {
        return (title.to_string(), None);
    }
    let clean_title = title[..start].trim().to_string();
    let author = title[start + 4..title.len() - 1].trim().to_string();
    (clean_title, (!author.is_empty()).then_some(author))
}

fn extract_markdown_image_url(line: &str) -> Option<String> {
    if !line.starts_with("![") {
        return None;
    }
    let start = line.find("](")? + 2;
    let end = line[start..].find(')')? + start;
    let url = line[start..end].trim();
    (!url.is_empty()).then(|| url.to_string())
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

fn source_id(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn source_name(url: &str) -> String {
    match resolve_import_adapter(url) {
        ImportAdapterKind::EvoLinkCases => "EvoLinkAI GPT Image 2 cases".to_string(),
        ImportAdapterKind::FreestyleflyGallery => "freestylefly GPT Image 2 gallery".to_string(),
        ImportAdapterKind::YouMindReadmeZh => "YouMind GPT Image 2 README_zh".to_string(),
        ImportAdapterKind::Generic => {
            parse_github_repo(url)
                .map(|repo| repo.name)
                .unwrap_or_else(|_| {
                    url.trim()
                        .trim_end_matches('/')
                        .rsplit('/')
                        .next()
                        .unwrap_or("Prompt Library")
                        .to_string()
                })
        }
    }
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
    if !contains_cjk(prompt) {
        return "en".to_string();
    }
    if contains_traditional_chinese_marker(prompt) {
        "zh-TW".to_string()
    } else {
        "zh-CN".to_string()
    }
}

fn is_simplified_chinese_prompt(prompt: &str) -> bool {
    contains_cjk(prompt) && !contains_traditional_chinese_marker(prompt)
}

fn contains_cjk(value: &str) -> bool {
    value
        .chars()
        .any(|char| ('\u{4e00}'..='\u{9fff}').contains(&char))
}

fn contains_traditional_chinese_marker(value: &str) -> bool {
    const TRADITIONAL_ONLY_CHARS: &[char] = &[
        '體', '圖', '畫', '顏', '風', '這', '個', '與', '為', '後', '裡', '讓', '將', '開', '關',
        '選', '標', '題', '視', '覺', '廣', '場', '燈', '優', '質', '創', '應', '時', '間', '層',
        '線', '邊', '貓', '隻',
    ];
    value
        .chars()
        .any(|char| TRADITIONAL_ONLY_CHARS.contains(&char))
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
    fn parses_github_tree_subdirectory_on_default_branch() {
        let selection = parse_github_tree_selection(
            "https://github.com/example/repo/tree/main/examples/prompts",
            "main",
        )
        .expect("tree selection should parse");

        assert_eq!(selection.branch.as_deref(), Some("main"));
        assert_eq!(selection.path_prefix.as_deref(), Some("examples/prompts"));
    }

    #[test]
    fn parses_github_tree_subdirectory_on_custom_branch() {
        let selection =
            parse_github_tree_selection("https://github.com/example/repo/tree/dev/prompts", "main")
                .expect("tree selection should parse");

        assert_eq!(selection.branch.as_deref(), Some("dev"));
        assert_eq!(selection.path_prefix.as_deref(), Some("prompts"));
    }

    #[test]
    fn parses_tree_url_with_slash_branch_and_directory() {
        let candidates = vec!["feature/import-fix".to_string()];
        let selection = parse_github_tree_selection_with_refs(
            "https://github.com/example/repo/tree/feature/import-fix/prompts",
            "main",
            &candidates,
        )
        .expect("tree selection should parse");

        assert_eq!(selection.branch.as_deref(), Some("feature/import-fix"));
        assert_eq!(selection.path_prefix.as_deref(), Some("prompts"));
    }

    #[test]
    fn parses_tree_url_with_deep_slash_branch_and_file_path() {
        let candidates = vec!["release/2026/05".to_string()];
        let selection = parse_github_tree_selection_with_refs(
            "https://github.com/example/repo/tree/release/2026/05/docs/a.md",
            "main",
            &candidates,
        )
        .expect("tree selection should parse");

        assert_eq!(selection.branch.as_deref(), Some("release/2026/05"));
        assert_eq!(selection.path_prefix.as_deref(), Some("docs/a.md"));
    }

    #[test]
    fn chooses_longest_branch_candidate_when_prefixes_overlap() {
        let candidates = vec!["feature".to_string(), "feature/import-fix".to_string()];
        let selection = parse_github_tree_selection_with_refs(
            "https://github.com/example/repo/tree/feature/import-fix/prompts",
            "main",
            &candidates,
        )
        .expect("tree selection should parse");

        assert_eq!(selection.branch.as_deref(), Some("feature/import-fix"));
        assert_eq!(selection.path_prefix.as_deref(), Some("prompts"));
    }

    #[test]
    fn converts_blob_url_with_slash_branch_to_raw_url() {
        let candidates = vec!["feature/import-fix".to_string()];
        let raw_url = github_blob_to_raw_with_refs(
            "https://github.com/example/repo/blob/feature/import-fix/prompts/a.md",
            &candidates,
        )
        .expect("blob URL should convert");

        assert_eq!(
            raw_url,
            "https://raw.githubusercontent.com/example/repo/feature/import-fix/prompts/a.md"
        );
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
        assert_eq!(
            items[0].negative_prompt.as_deref(),
            Some("blurry, watermark")
        );
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
        assert_eq!(
            items[0].prompt_original,
            "A premium perfume bottle on reflective glass."
        );
        assert_eq!(
            items[0].tags,
            vec!["product".to_string(), "poster".to_string()]
        );
    }

    #[test]
    fn extracts_prompt_from_fenced_code_block_with_image_and_author() {
        let markdown = r#"
## Character

### Neon Samurai (by @artist)

![preview](https://example.com/neon.png)

```prompt
A neon samurai standing in the rain, cinematic lighting.
```

Negative Prompt: low quality
"#;

        let items = parse_prompt_document(
            "https://github.com/example/repo",
            "https://raw.githubusercontent.com/example/repo/main/README.md",
            markdown,
        )
        .expect("markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Neon Samurai");
        assert_eq!(items[0].author.as_deref(), Some("@artist"));
        assert_eq!(
            items[0].prompt_original,
            "A neon samurai standing in the rain, cinematic lighting."
        );
        assert_eq!(
            items[0].preview_image_urls,
            vec!["https://example.com/neon.png".to_string()]
        );
    }

    #[test]
    fn ignores_unlabeled_install_code_blocks() {
        let markdown = r#"
## Quick Install the Skill

```
codex skill install imagegen
```

## Examples

### Poster Prompt

```prompt
A cinematic poster for a tea brand.
```
"#;

        let items = parse_prompt_document(
            "https://github.com/example/repo",
            "https://raw.githubusercontent.com/example/repo/main/README.md",
            markdown,
        )
        .expect("markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Poster Prompt");
        assert_eq!(
            items[0].prompt_original,
            "A cinematic poster for a tea brand."
        );
    }

    #[test]
    fn extracts_multiple_prompt_cases_in_one_category() {
        let markdown = r#"
## Posters

### Perfume Ad
Prompt: A luxury perfume poster with reflective glass.

### Coffee Ad
Prompt: A warm coffee shop poster with morning light.
"#;

        let items = parse_prompt_document(
            "https://github.com/example/repo",
            "https://raw.githubusercontent.com/example/repo/main/README.md",
            markdown,
        )
        .expect("markdown should parse");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Perfume Ad");
        assert_eq!(items[1].title, "Coffee Ad");
        assert_eq!(items[1].category, "Posters");
    }

    #[test]
    fn normalizes_default_gpt_image_2_repo_root_to_cases_tree() {
        let info = classify_import_url(
            "https://github.com/EvoLinkAI/awesome-gpt-image-2-API-and-Prompts/?tab=readme#readme",
        );

        assert_eq!(info.source_type, "github_tree");
        assert_eq!(info.normalized_url, DEFAULT_GPT_IMAGE_2_CASES_URL);
    }

    #[test]
    fn keeps_default_gpt_image_2_cases_tree_url_unchanged() {
        let info = classify_import_url(DEFAULT_GPT_IMAGE_2_CASES_URL);

        assert_eq!(info.source_type, "github_tree");
        assert_eq!(info.normalized_url, DEFAULT_GPT_IMAGE_2_CASES_URL);
    }

    #[test]
    fn does_not_rewrite_other_github_repo_roots() {
        let info = classify_import_url("https://github.com/example/repo");

        assert_eq!(info.source_type, "github_repo");
        assert_eq!(info.normalized_url, "https://github.com/example/repo");
    }

    #[test]
    fn prefers_simplified_chinese_items_for_default_cases_source() {
        let items = vec![
            build_prompt_draft(
                "https://github.com/example/repo",
                "https://raw.githubusercontent.com/example/repo/main/cases/zh.md",
                "简体中文",
                "cases",
                "一只橙色小猫坐在窗边，柔和自然光。".to_string(),
                None,
                None,
                Vec::new(),
            ),
            build_prompt_draft(
                "https://github.com/example/repo",
                "https://raw.githubusercontent.com/example/repo/main/cases/tw.md",
                "繁體中文",
                "cases",
                "一隻橙色小貓坐在窗邊，柔和自然光。".to_string(),
                None,
                None,
                Vec::new(),
            ),
            build_prompt_draft(
                "https://github.com/example/repo",
                "https://raw.githubusercontent.com/example/repo/main/cases/en.md",
                "English",
                "cases",
                "An orange kitten sitting by the window.".to_string(),
                None,
                None,
                Vec::new(),
            ),
        ];

        let filtered = prefer_simplified_chinese_items(items);

        assert_eq!(filtered.len(), 1);
        assert_eq!(
            filtered[0].prompt_original,
            "一只橙色小猫坐在窗边，柔和自然光。"
        );
        assert_eq!(filtered[0].language, "zh-CN");
    }

    #[test]
    fn falls_back_to_all_items_when_no_simplified_chinese_items_exist() {
        let items = vec![
            build_prompt_draft(
                "https://github.com/example/repo",
                "https://raw.githubusercontent.com/example/repo/main/cases/tw.md",
                "繁體中文",
                "cases",
                "一隻橙色小貓坐在窗邊，柔和自然光。".to_string(),
                None,
                None,
                Vec::new(),
            ),
            build_prompt_draft(
                "https://github.com/example/repo",
                "https://raw.githubusercontent.com/example/repo/main/cases/en.md",
                "English",
                "cases",
                "An orange kitten sitting by the window.".to_string(),
                None,
                None,
                Vec::new(),
            ),
        ];

        let filtered = prefer_simplified_chinese_items(items);

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn cleans_prompt_text_before_building_markdown_item() {
        let markdown = r#"
## 人像

### 电影感少女

提示词：
```text
1. **一位穿红色斗篷的少女站在雪山之巅，电影级光影，细腻面部表情。**
```
"#;

        let items = parse_prompt_document(
            "https://github.com/example/repo",
            "https://raw.githubusercontent.com/example/repo/main/cases/zh.md",
            markdown,
        )
        .expect("markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].prompt_original,
            "一位穿红色斗篷的少女站在雪山之巅，电影级光影，细腻面部表情。"
        );
    }

    #[test]
    fn cleans_prompt_text_from_json_prompt_field() {
        let json = r#"
[
  {
    "title": "产品海报",
    "prompt": "提示词：- **一瓶高级香水置于反光玻璃上，柔和棚拍灯光。**"
  }
]
"#;

        let items = parse_prompt_document(
            "https://github.com/example/repo",
            "https://raw.githubusercontent.com/example/repo/main/cases/prompts.json",
            json,
        )
        .expect("json should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].prompt_original,
            "一瓶高级香水置于反光玻璃上，柔和棚拍灯光。"
        );
    }

    #[test]
    fn removes_metadata_lines_from_prompt_text() {
        let cleaned = clean_prompt_text(
            r#"
标题：产品海报
分类：商业摄影
提示词：一瓶高级香水置于反光玻璃上，柔和棚拍灯光。
比例：1:1
负面提示词：低清晰度，水印
"#,
        );

        assert_eq!(cleaned, "一瓶高级香水置于反光玻璃上，柔和棚拍灯光。");
    }

    #[test]
    fn normalizes_freestylefly_gallery_builtin_url() {
        let info = classify_import_url(
            "https://github.com/freestylefly/awesome-gpt-image-2/blob/main/docs/gallery.md?plain=1",
        );

        assert_eq!(info.source_type, "github_blob");
        assert_eq!(info.normalized_url, FREESTYLEFLY_GALLERY_URL);
    }

    #[test]
    fn normalizes_youmind_readme_zh_builtin_url() {
        let info = classify_import_url(
            "https://raw.githubusercontent.com/YouMind-OpenLab/awesome-gpt-image-2/main/README_zh.md",
        );

        assert_eq!(info.source_type, "github_blob");
        assert_eq!(info.normalized_url, YOUMIND_README_ZH_URL);
    }

    #[test]
    fn resolves_builtin_source_adapters() {
        assert_eq!(
            resolve_import_adapter(DEFAULT_GPT_IMAGE_2_CASES_URL),
            ImportAdapterKind::EvoLinkCases
        );
        assert_eq!(
            resolve_import_adapter(FREESTYLEFLY_GALLERY_URL),
            ImportAdapterKind::FreestyleflyGallery
        );
        assert_eq!(
            resolve_import_adapter(YOUMIND_README_ZH_URL),
            ImportAdapterKind::YouMindReadmeZh
        );
        assert_eq!(
            resolve_import_adapter("https://github.com/example/repo"),
            ImportAdapterKind::Generic
        );
    }

    #[test]
    fn extracts_freestylefly_gallery_part_links_from_index() {
        let markdown = r#"
- [Part 1：例 1-165](./gallery-part-1.md)
- [Part 2：例 166-427](./gallery-part-2.md)
- [Part 1 duplicate](./gallery-part-1.md)
"#;

        let links = extract_freestylefly_gallery_part_links(markdown);

        assert_eq!(
            links,
            vec![
                FREESTYLEFLY_PART_1_RAW_URL.to_string(),
                FREESTYLEFLY_PART_2_RAW_URL.to_string()
            ]
        );
    }

    #[test]
    fn parses_freestylefly_gallery_case_preferring_chinese_block() {
        let markdown = r#"
### 例 166：十二黄金圣斗士卡牌合集

![十二黄金圣斗士卡牌合集](../data/images/case166.jpg)

**来源：** someone

**提示词：**

```text
[中文]
生成圣斗士星矢12个黄金圣斗士的12宫格卡牌图片。

[English]
Generate a 12-grid card image of the 12 Gold Saints.
```
"#;

        let items = parse_prompt_document_for_adapter(
            ImportAdapterKind::FreestyleflyGallery,
            FREESTYLEFLY_GALLERY_URL,
            FREESTYLEFLY_PART_2_RAW_URL,
            markdown,
        )
        .expect("freestylefly markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "十二黄金圣斗士卡牌合集");
        assert_eq!(
            items[0].prompt_original,
            "生成圣斗士星矢12个黄金圣斗士的12宫格卡牌图片。"
        );
        assert!(!items[0].prompt_original.contains("English"));
        assert_eq!(
            items[0].preview_image_urls,
            vec!["../data/images/case166.jpg".to_string()]
        );
    }

    #[test]
    fn parses_freestylefly_gallery_case_without_language_markers() {
        let markdown = r#"
### 例 1：信息图可视化设计

**提示词：**

```
Vertical 9:16 isometric cutaway infographic.
```
"#;

        let items = parse_prompt_document_for_adapter(
            ImportAdapterKind::FreestyleflyGallery,
            FREESTYLEFLY_GALLERY_URL,
            FREESTYLEFLY_PART_1_RAW_URL,
            markdown,
        )
        .expect("freestylefly markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].prompt_original,
            "Vertical 9:16 isometric cutaway infographic."
        );
    }

    #[test]
    fn parses_youmind_readme_zh_prompt_json_as_raw_prompt() {
        let markdown = r#"
### No. 1: VR 头显爆炸视图海报

#### 📖 描述

生成一张高科技 VR 头显爆炸视图。

#### 📝 提示词

```json
{
  "type": "产品爆炸视图海报",
  "subject": "VR 头显"
}
```

#### 🖼️ 生成图片

<img src="https://example.com/vr.jpg" width="700" alt="VR">

#### 📌 详情

- **作者:** someone
"#;

        let items = parse_prompt_document_for_adapter(
            ImportAdapterKind::YouMindReadmeZh,
            YOUMIND_README_ZH_URL,
            "https://raw.githubusercontent.com/YouMind-OpenLab/awesome-gpt-image-2/main/README_zh.md",
            markdown,
        )
        .expect("youmind markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "VR 头显爆炸视图海报");
        assert!(items[0]
            .prompt_original
            .contains("\"type\": \"产品爆炸视图海报\""));
        assert!(!items[0].prompt_original.contains("作者"));
        assert_eq!(
            items[0].preview_image_urls,
            vec!["https://example.com/vr.jpg".to_string()]
        );
    }

    #[test]
    fn parses_multiple_youmind_cases() {
        let markdown = r#"
### No. 1: 第一个案例

#### 📝 提示词

```
第一个提示词。
```

### No. 2: 第二个案例

#### 📝 提示词

```
第二个提示词。
```
"#;

        let items = parse_prompt_document_for_adapter(
            ImportAdapterKind::YouMindReadmeZh,
            YOUMIND_README_ZH_URL,
            "https://raw.githubusercontent.com/YouMind-OpenLab/awesome-gpt-image-2/main/README_zh.md",
            markdown,
        )
        .expect("youmind markdown should parse");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "第一个案例");
        assert_eq!(items[1].title, "第二个案例");
    }

    #[test]
    fn captures_youmind_preview_images_outside_prompt() {
        let markdown = r#"
### No. 3: 图片案例

#### 📝 提示词

```
生成一张图片。
```

![preview](https://example.com/preview.png)
"#;

        let items = parse_prompt_document_for_adapter(
            ImportAdapterKind::YouMindReadmeZh,
            YOUMIND_README_ZH_URL,
            "https://raw.githubusercontent.com/YouMind-OpenLab/awesome-gpt-image-2/main/README_zh.md",
            markdown,
        )
        .expect("youmind markdown should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].preview_image_urls,
            vec!["https://example.com/preview.png".to_string()]
        );
        assert!(!items[0].prompt_original.contains("preview.png"));
    }

    #[test]
    fn evolink_cases_adapter_uses_generic_parser_and_simplified_post_process() {
        let markdown = r#"
## Cases

### 简体案例
提示词：一只橙色小猫坐在窗边。

### English Case
Prompt: An orange kitten sitting by the window.
"#;

        let items = parse_prompt_document_for_adapter(
            ImportAdapterKind::EvoLinkCases,
            DEFAULT_GPT_IMAGE_2_CASES_URL,
            "https://raw.githubusercontent.com/EvoLinkAI/awesome-gpt-image-2-API-and-Prompts/main/cases/demo.md",
            markdown,
        )
        .expect("evolink markdown should parse");
        let items = post_process_import_items(ImportAdapterKind::EvoLinkCases, items);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "简体案例");
    }
}
