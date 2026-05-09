use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub url: String,
    pub model: String,
    pub user_content: String,
    pub api_key: String,
    pub provider: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptOptimizationResult {
    pub prompt: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    content: String,
}

pub fn optimize_prompt(config: &AppConfig, local_prompt: &str) -> Result<PromptOptimizationResult, String> {
    let request = build_prompt_request(config, local_prompt)?;
    let optimized = if request.provider == "claude" {
        call_claude(&request)?
    } else {
        call_openai_compatible(&request)?
    };
    Ok(PromptOptimizationResult { prompt: optimized })
}

pub fn build_prompt_request(config: &AppConfig, local_prompt: &str) -> Result<PromptRequest, String> {
    let provider = &config.prompt_optimization;
    if !provider.enabled || provider.provider == "local-rules" {
        return Err("提示词优化 API 未启用。".to_string());
    }
    if provider.api_key.trim().is_empty() {
        return Err("提示词优化 API Key 为空。".to_string());
    }
    if provider.model.trim().is_empty() {
        return Err("提示词优化模型 ID 为空。".to_string());
    }

    let provider_id = provider.provider.trim();
    let url = match provider_id {
        "openai" => if provider.base_url.trim().is_empty() {
            "https://api.openai.com/v1/chat/completions".to_string()
        } else {
            format!("{}/chat/completions", provider.base_url.trim().trim_end_matches('/'))
        },
        "claude" => if provider.base_url.trim().is_empty() {
            "https://api.anthropic.com/v1/messages".to_string()
        } else {
            format!("{}/messages", provider.base_url.trim().trim_end_matches('/'))
        },
        "compatible" => {
            if provider.base_url.trim().is_empty() {
                return Err("自定义提示词优化 API 需要填写 Base URL。".to_string());
            }
            format!("{}/chat/completions", provider.base_url.trim().trim_end_matches('/'))
        }
        _ => return Err(format!("不支持的提示词优化 Provider: {provider_id}")),
    };

    Ok(PromptRequest {
        url,
        model: provider.model.trim().to_string(),
        user_content: build_user_content(local_prompt),
        api_key: provider.api_key.trim().to_string(),
        provider: provider_id.to_string(),
    })
}

fn build_user_content(local_prompt: &str) -> String {
    format!(
        "请将下面的图像生成提示词优化为更专业、可直接用于图像生成的版本。保留中文表达，同时让画面主体、场景、构图、光线、风格和负面约束更清晰。只输出优化后的提示词，不要解释。\n\n{}",
        local_prompt.trim()
    )
}

fn call_openai_compatible(request: &PromptRequest) -> Result<String, String> {
    let body = json!({
        "model": request.model,
        "messages": [
            {
                "role": "system",
                "content": "You are a prompt optimizer for image generation."
            },
            {
                "role": "user",
                "content": request.user_content
            }
        ],
        "temperature": 0.7
    });
    let response = ureq::post(&request.url)
        .header("Authorization", &format!("Bearer {}", request.api_key))
        .header("Content-Type", "application/json")
        .send_json(body)
        .map_err(|err| format!("提示词优化 API 调用失败: {err}"))?;
    let parsed: OpenAiChatResponse = response
        .into_body()
        .read_json()
        .map_err(|err| format!("提示词优化 API 响应解析失败: {err}"))?;
    parsed
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "提示词优化 API 没有返回内容。".to_string())
}

fn call_claude(request: &PromptRequest) -> Result<String, String> {
    let body = json!({
        "model": request.model,
        "max_tokens": 1200,
        "messages": [
            {
                "role": "user",
                "content": request.user_content
            }
        ]
    });
    let response = ureq::post(&request.url)
        .header("x-api-key", &request.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .send_json(body)
        .map_err(|err| format!("Claude 提示词优化 API 调用失败: {err}"))?;
    let parsed: Value = response
        .into_body()
        .read_json()
        .map_err(|err| format!("Claude 响应解析失败: {err}"))?;
    parsed
        .get("content")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| "Claude 没有返回提示词内容。".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiProviderConfig, AppConfig};

    #[test]
    fn rejects_disabled_prompt_optimization_config() {
        let config = AppConfig::default();

        let error = build_prompt_request(&config, "local prompt").expect_err("disabled config should fail");

        assert!(error.contains("提示词优化 API 未启用"));
    }

    #[test]
    fn builds_openai_compatible_prompt_request() {
        let mut config = AppConfig::default();
        config.prompt_optimization = ApiProviderConfig {
            enabled: true,
            provider: "compatible".to_string(),
            base_url: "https://llm.example.com/v1".to_string(),
            model: "custom-model".to_string(),
            api_key: "sk-test".to_string(),
        };

        let request = build_prompt_request(&config, "一只猫，电影感").expect("request should build");

        assert_eq!(request.url, "https://llm.example.com/v1/chat/completions");
        assert_eq!(request.model, "custom-model");
        assert!(request.user_content.contains("一只猫"));
    }
}
