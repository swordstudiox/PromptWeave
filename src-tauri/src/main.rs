#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod db;
mod generation;
mod imports;
mod prompt_api;
mod providers;
mod workspace;

#[tauri::command]
async fn init_workspace() -> Result<workspace::WorkspaceInfo, String> {
    run_blocking_command(|| {
        let root = workspace::default_workspace_root()?;
        let info = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&info.database_path))?;
        Ok(info)
    })
    .await
}

#[tauri::command]
fn classify_import_url(url: String) -> imports::ImportUrlInfo {
    imports::classify_import_url(&url)
}

#[tauri::command]
async fn preview_import_url(url: String) -> Result<imports::ImportPreview, String> {
    run_blocking_command(move || imports::preview_import_url(&url)).await
}

#[tauri::command]
async fn import_prompt_library(url: String) -> Result<imports::ImportResult, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        imports::import_prompt_library(&root, &url)
    })
    .await
}

#[tauri::command]
async fn list_prompt_library_sources() -> Result<Vec<db::PromptLibrarySourceRecord>, String> {
    run_blocking_command(|| {
        let root = workspace::default_workspace_root()?;
        imports::list_prompt_library_sources(&root)
    })
    .await
}

#[tauri::command]
async fn sync_prompt_library_source(source_id: String) -> Result<imports::ImportResult, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        imports::sync_prompt_library_source(&root, &source_id)
    })
    .await
}

async fn run_blocking_command<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| format!("Background task failed: {err}"))?
}

#[tauri::command]
async fn list_prompt_templates(limit: usize) -> Result<Vec<db::PromptTemplateRecord>, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let workspace = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&workspace.database_path))?;
        db::list_prompt_templates(std::path::Path::new(&workspace.database_path), limit)
    })
    .await
}

#[tauri::command]
async fn search_prompt_templates(
    query: String,
    limit: usize,
) -> Result<Vec<db::PromptTemplateRecord>, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let workspace = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&workspace.database_path))?;
        db::search_prompt_templates(
            std::path::Path::new(&workspace.database_path),
            &query,
            limit,
        )
    })
    .await
}

#[tauri::command]
async fn update_prompt_template(draft: db::TemplateUpdateDraft) -> Result<(), String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let workspace = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&workspace.database_path))?;
        db::update_prompt_template(std::path::Path::new(&workspace.database_path), &draft)
    })
    .await
}

#[tauri::command]
async fn toggle_prompt_template_favorite(id: String, is_favorite: bool) -> Result<(), String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let workspace = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&workspace.database_path))?;
        db::toggle_prompt_template_favorite(
            std::path::Path::new(&workspace.database_path),
            &id,
            is_favorite,
        )
    })
    .await
}

#[tauri::command]
async fn archive_prompt_template(id: String) -> Result<(), String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let workspace = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&workspace.database_path))?;
        db::archive_prompt_template(std::path::Path::new(&workspace.database_path), &id)
    })
    .await
}

#[tauri::command]
async fn save_generation_history(draft: db::GenerationHistoryDraft) -> Result<(), String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let workspace = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&workspace.database_path))?;
        db::save_generation_history(std::path::Path::new(&workspace.database_path), &draft)
    })
    .await
}

#[tauri::command]
async fn list_generation_history(limit: usize) -> Result<Vec<db::GenerationHistoryRecord>, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let workspace = workspace::ensure_workspace(&root)?;
        db::bootstrap(std::path::Path::new(&workspace.database_path))?;
        db::list_generation_history(std::path::Path::new(&workspace.database_path), limit)
    })
    .await
}

#[tauri::command]
async fn get_app_config() -> Result<config::AppConfig, String> {
    run_blocking_command(|| {
        let root = workspace::default_workspace_root()?;
        workspace::ensure_workspace(&root)?;
        config::load_config(&root)
    })
    .await
}

#[tauri::command]
async fn save_app_config(config: config::AppConfig) -> Result<config::AppConfig, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        config::save_config(&root, &config)
    })
    .await
}

#[tauri::command]
async fn generate_image_preview(
    prompt: String,
    options: generation::ImageGenerationOptions,
) -> Result<generation::ImageGenerationResult, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let config = config::load_config(&root)?;
        generation::generate_image(&root, &config, &prompt, &options)
    })
    .await
}

#[tauri::command]
async fn optimize_prompt_with_api(
    local_prompt: String,
) -> Result<prompt_api::PromptOptimizationResult, String> {
    run_blocking_command(move || {
        let root = workspace::default_workspace_root()?;
        let config = config::load_config(&root)?;
        prompt_api::optimize_prompt(&config, &local_prompt)
    })
    .await
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            init_workspace,
            classify_import_url,
            preview_import_url,
            import_prompt_library,
            list_prompt_library_sources,
            sync_prompt_library_source,
            list_prompt_templates,
            search_prompt_templates,
            update_prompt_template,
            toggle_prompt_template_favorite,
            archive_prompt_template,
            save_generation_history,
            list_generation_history,
            get_app_config,
            save_app_config,
            generate_image_preview,
            optimize_prompt_with_api
        ])
        .run(tauri::generate_context!())
        .expect("failed to run PromptWeave");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocking_command_helper_returns_success() {
        let result = tauri::async_runtime::block_on(run_blocking_command(|| {
            Ok::<_, String>("ready".to_string())
        }));

        assert_eq!(result.expect("task should succeed"), "ready");
    }

    #[test]
    fn blocking_command_helper_returns_task_error() {
        let result = tauri::async_runtime::block_on(run_blocking_command(|| {
            Err::<String, _>("network failed".to_string())
        }));

        assert_eq!(
            result.expect_err("task error should propagate"),
            "network failed"
        );
    }
}
