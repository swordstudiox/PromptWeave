use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUrlInfo {
    pub source_type: String,
    pub normalized_url: String,
    pub is_supported: bool,
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
