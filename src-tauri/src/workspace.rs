use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub root: String,
    pub data_dir: String,
    pub database_path: String,
}

pub fn ensure_workspace(root: &Path) -> Result<WorkspaceInfo, String> {
    let data_dir = root.join(".promptweave");
    let cache_dir = data_dir.join("cache");
    let imports_dir = data_dir.join("imports");
    let exports_dir = data_dir.join("exports");
    let history_dir = data_dir.join("history");

    for dir in [&data_dir, &cache_dir, &imports_dir, &exports_dir, &history_dir] {
        fs::create_dir_all(dir).map_err(|err| format!("Failed to create {}: {err}", dir.display()))?;
    }

    let database_path = data_dir.join("db.sqlite");

    Ok(WorkspaceInfo {
        root: root.display().to_string(),
        data_dir: data_dir.display().to_string(),
        database_path: database_path.display().to_string(),
    })
}

pub fn default_workspace_root() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|err| format!("Failed to resolve current directory: {err}"))
}
