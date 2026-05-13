use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct ImageRequest {
    pub url: String,
    pub model: String,
    pub prompt: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationResult {
    pub image_path: Option<String>,
    pub image_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationOptions {
    pub size: String,
    pub quality: String,
    pub n: u32,
}

#[derive(Debug, Deserialize)]
struct ImageApiResponse {
    data: Vec<ImageApiData>,
}

#[derive(Debug, Deserialize)]
struct ImageApiData {
    b64_json: Option<String>,
    url: Option<String>,
}

pub fn generate_image(
    workspace_root: &Path,
    config: &AppConfig,
    prompt: &str,
    options: &ImageGenerationOptions,
) -> Result<ImageGenerationResult, String> {
    let request = build_image_request(config, prompt)?;
    let body = json!({
        "model": request.model,
        "prompt": request.prompt,
        "size": options.size,
        "quality": options.quality,
        "n": options.n.clamp(1, 4)
    });
    let response = ureq::post(&request.url)
        .header("Authorization", &format!("Bearer {}", request.api_key))
        .header("Content-Type", "application/json")
        .send_json(body)
        .map_err(|err| format!("图片生成 API 调用失败: {err}"))?;
    let parsed: ImageApiResponse = response
        .into_body()
        .read_json()
        .map_err(|err| format!("图片生成 API 响应解析失败: {err}"))?;
    if parsed.data.is_empty() {
        return Err("图片生成 API 没有返回图片。".to_string());
    }

    let mut image_paths = Vec::new();
    for image in parsed.data {
        let bytes = if let Some(encoded) = &image.b64_json {
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .map_err(|err| format!("图片 base64 解码失败: {err}"))?
        } else if let Some(url) = &image.url {
            ureq::get(url)
                .call()
                .map_err(|err| format!("图片 URL 下载失败: {err}"))?
                .into_body()
                .read_to_vec()
                .map_err(|err| format!("图片内容读取失败: {err}"))?
        } else {
            return Err("图片生成 API 响应缺少 b64_json 或 url。".to_string());
        };
        image_paths.push(save_image_bytes(workspace_root, &bytes)?.display().to_string());
    }

    Ok(ImageGenerationResult {
        image_path: image_paths.first().cloned(),
        image_paths,
    })
}

pub fn build_image_request(config: &AppConfig, prompt: &str) -> Result<ImageRequest, String> {
    let provider = &config.image_generation;
    if !provider.enabled || provider.provider == "disabled" {
        return Err("图片生成 API 未启用。".to_string());
    }
    if provider.api_key.trim().is_empty() {
        return Err("图片生成 API Key 为空。".to_string());
    }

    let default_url = if provider.provider == "gpt-image" {
        "https://api.openai.com/v1/images/generations"
    } else {
        ""
    };
    let url = if provider.base_url.trim().is_empty() {
        default_url.to_string()
    } else {
        provider.base_url.trim().trim_end_matches('/').to_string()
    };
    if url.is_empty() {
        return Err("自定义图片生成 API 需要填写 Base URL。".to_string());
    }

    let model = if provider.model.trim().is_empty() {
        "gpt-image-1.5".to_string()
    } else {
        provider.model.trim().to_string()
    };

    Ok(ImageRequest {
        url,
        model,
        prompt: prompt.trim().to_string(),
        api_key: provider.api_key.trim().to_string(),
    })
}

fn save_image_bytes(workspace_root: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let workspace = crate::workspace::ensure_workspace(workspace_root)?;
    let image_dir = Path::new(&workspace.data_dir).join("history").join("images");
    fs::create_dir_all(&image_dir)
        .map_err(|err| format!("图片历史目录创建失败 {}: {err}", image_dir.display()))?;
    let path = image_dir.join(format!("image-{}-{}.png", current_timestamp(), random_suffix()));
    fs::write(&path, bytes).map_err(|err| format!("图片保存失败 {}: {err}", path.display()))?;
    Ok(path)
}

fn random_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
}

fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiProviderConfig, AppConfig};

    #[test]
    fn rejects_disabled_image_generation_config() {
        let config = AppConfig::default();

        let error = build_image_request(&config, "a cat").expect_err("disabled config should fail");

        assert!(error.contains("图片生成 API 未启用"));
    }

    #[test]
    fn builds_default_gpt_image_request() {
        let mut config = AppConfig::default();
        config.image_generation = ApiProviderConfig {
            enabled: true,
            provider: "gpt-image".to_string(),
            base_url: String::new(),
            model: String::new(),
            api_key: "sk-test".to_string(),
        };

        let request = build_image_request(&config, "a cinematic cat").expect("request should build");

        assert_eq!(request.url, "https://api.openai.com/v1/images/generations");
        assert_eq!(request.model, "gpt-image-1.5");
        assert_eq!(request.prompt, "a cinematic cat");
    }
}
