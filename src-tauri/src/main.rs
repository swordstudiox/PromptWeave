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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![init_workspace, classify_import_url])
        .run(tauri::generate_context!())
        .expect("failed to run PromptWeave");
}
