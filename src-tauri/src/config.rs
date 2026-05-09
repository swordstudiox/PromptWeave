use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub prompt_optimization: ApiProviderConfig,
    pub image_generation: ApiProviderConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiProviderConfig {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompt_optimization: ApiProviderConfig {
                enabled: false,
                provider: "local-rules".to_string(),
                base_url: String::new(),
                model: String::new(),
                api_key: String::new(),
            },
            image_generation: ApiProviderConfig {
                enabled: false,
                provider: "disabled".to_string(),
                base_url: String::new(),
                model: String::new(),
                api_key: String::new(),
            },
        }
    }
}

pub fn load_config(workspace_root: &Path) -> Result<AppConfig, String> {
    let config_path = config_path(workspace_root);
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|err| format!("Failed to read config {}: {err}", config_path.display()))?;
    serde_json::from_str(&content).map_err(|err| format!("Invalid config {}: {err}", config_path.display()))
}

pub fn save_config(workspace_root: &Path, config: &AppConfig) -> Result<AppConfig, String> {
    let workspace = crate::workspace::ensure_workspace(workspace_root)?;
    let config_path = Path::new(&workspace.data_dir).join("config.json");
    let content = serde_json::to_string_pretty(config).map_err(|err| format!("Failed to serialize config: {err}"))?;
    fs::write(&config_path, content).map_err(|err| format!("Failed to write config {}: {err}", config_path.display()))?;
    Ok(config.clone())
}

fn config_path(workspace_root: &Path) -> std::path::PathBuf {
    workspace_root.join(".promptweave").join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_workspace(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("promptweave-config-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        path
    }

    #[test]
    fn returns_default_config_when_file_is_missing() {
        let root = temp_workspace("default");

        let config = load_config(&root).expect("default config should load");

        assert_eq!(config.prompt_optimization.provider, "local-rules");
        assert!(!config.image_generation.enabled);
        assert_eq!(config.image_generation.provider, "disabled");
    }

    #[test]
    fn saves_and_loads_workspace_config() {
        let root = temp_workspace("roundtrip");
        let mut config = AppConfig::default();
        config.prompt_optimization.enabled = true;
        config.prompt_optimization.provider = "openai".to_string();
        config.prompt_optimization.model = "gpt-5.2".to_string();
        config.prompt_optimization.api_key = "sk-test".to_string();
        config.image_generation.enabled = true;
        config.image_generation.provider = "gpt-image".to_string();
        config.image_generation.model = "gpt-image-2".to_string();

        save_config(&root, &config).expect("config should save");
        let loaded = load_config(&root).expect("config should load");

        assert_eq!(loaded.prompt_optimization.provider, "openai");
        assert_eq!(loaded.prompt_optimization.api_key, "sk-test");
        assert!(loaded.image_generation.enabled);
        assert_eq!(loaded.image_generation.model, "gpt-image-2");
    }
}
