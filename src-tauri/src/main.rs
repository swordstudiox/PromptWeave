mod db;
mod imports;
mod providers;
mod workspace;

#[tauri::command]
fn init_workspace() -> Result<workspace::WorkspaceInfo, String> {
    let root = workspace::default_workspace_root()?;
    let info = workspace::ensure_workspace(&root)?;
    db::bootstrap(std::path::Path::new(&info.database_path))?;
    Ok(info)
}

#[tauri::command]
fn classify_import_url(url: String) -> imports::ImportUrlInfo {
    imports::classify_import_url(&url)
}

#[tauri::command]
fn preview_import_url(url: String) -> Result<imports::ImportPreview, String> {
    imports::preview_import_url(&url)
}

#[tauri::command]
fn import_prompt_library(url: String) -> Result<imports::ImportResult, String> {
    let root = workspace::default_workspace_root()?;
    imports::import_prompt_library(&root, &url)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            init_workspace,
            classify_import_url,
            preview_import_url,
            import_prompt_library
        ])
        .run(tauri::generate_context!())
        .expect("failed to run PromptWeave");
}
