mod config;
mod db;
mod generation;
mod imports;
mod prompt_api;
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

#[tauri::command]
fn list_prompt_templates(limit: usize) -> Result<Vec<db::PromptTemplateRecord>, String> {
    let root = workspace::default_workspace_root()?;
    let workspace = workspace::ensure_workspace(&root)?;
    db::bootstrap(std::path::Path::new(&workspace.database_path))?;
    db::list_prompt_templates(std::path::Path::new(&workspace.database_path), limit)
}

#[tauri::command]
fn search_prompt_templates(query: String, limit: usize) -> Result<Vec<db::PromptTemplateRecord>, String> {
    let root = workspace::default_workspace_root()?;
    let workspace = workspace::ensure_workspace(&root)?;
    db::bootstrap(std::path::Path::new(&workspace.database_path))?;
    db::search_prompt_templates(std::path::Path::new(&workspace.database_path), &query, limit)
}

#[tauri::command]
fn get_app_config() -> Result<config::AppConfig, String> {
    let root = workspace::default_workspace_root()?;
    workspace::ensure_workspace(&root)?;
    config::load_config(&root)
}

#[tauri::command]
fn save_app_config(config: config::AppConfig) -> Result<config::AppConfig, String> {
    let root = workspace::default_workspace_root()?;
    config::save_config(&root, &config)
}

#[tauri::command]
fn generate_image_preview(
    prompt: String,
    options: generation::ImageGenerationOptions,
) -> Result<generation::ImageGenerationResult, String> {
    let root = workspace::default_workspace_root()?;
    let config = config::load_config(&root)?;
    generation::generate_image(&root, &config, &prompt, &options)
}

#[tauri::command]
fn optimize_prompt_with_api(local_prompt: String) -> Result<prompt_api::PromptOptimizationResult, String> {
    let root = workspace::default_workspace_root()?;
    let config = config::load_config(&root)?;
    prompt_api::optimize_prompt(&config, &local_prompt)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            init_workspace,
            classify_import_url,
            preview_import_url,
            import_prompt_library,
            list_prompt_templates,
            search_prompt_templates,
            get_app_config,
            save_app_config,
            generate_image_preview,
            optimize_prompt_with_api
        ])
        .run(tauri::generate_context!())
        .expect("failed to run PromptWeave");
}
