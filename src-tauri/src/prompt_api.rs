use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub url: String,
    pub fallback_url: Option<String>,
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

struct ApiResponseText {
    status: u16,
    content_type: String,
    body: String,
}

pub fn optimize_prompt(
    config: &AppConfig,
    local_prompt: &str,
) -> Result<PromptOptimizationResult, String> {
    let request = build_prompt_request(config, local_prompt)?;
    let optimized = match request.provider.as_str() {
        "claude" => call_claude(&request)?,
        "compatible" => call_compatible_with_fallback(&request)?,
        _ => call_openai_compatible(&request)?,
    };
    Ok(PromptOptimizationResult { prompt: optimized })
}

pub fn build_prompt_request(
    config: &AppConfig,
    local_prompt: &str,
) -> Result<PromptRequest, String> {
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
    let (url, fallback_url) = match provider_id {
        "openai" => (
            build_endpoint_url(
                &provider.base_url,
                "https://api.openai.com/v1/chat/completions",
                "chat/completions",
            ),
            None,
        ),
        "claude" => (
            build_endpoint_url(
                &provider.base_url,
                "https://api.anthropic.com/v1/messages",
                "messages",
            ),
            None,
        ),
        "compatible" => {
            if provider.base_url.trim().is_empty() {
                return Err("自定义提示词优化 API 需要填写 Base URL。".to_string());
            }
            (
                build_endpoint_url(&provider.base_url, "", "messages"),
                Some(build_endpoint_url(
                    &provider.base_url,
                    "",
                    "chat/completions",
                )),
            )
        }
        _ => return Err(format!("不支持的提示词优化 Provider: {provider_id}")),
    };

    Ok(PromptRequest {
        url,
        fallback_url,
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

fn build_endpoint_url(base_url: &str, default_url: &str, endpoint_path: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return default_url.to_string();
    }

    let endpoint_path = endpoint_path.trim_matches('/');
    if base.ends_with(&format!("/{endpoint_path}")) {
        return base.to_string();
    }

    let base = strip_known_endpoint_suffix(base);
    let versioned_base = if base_has_path(base) {
        base.to_string()
    } else {
        format!("{base}/v1")
    };
    format!("{versioned_base}/{endpoint_path}")
}

fn strip_known_endpoint_suffix(base_url: &str) -> &str {
    for suffix in ["/chat/completions", "/messages"] {
        if let Some(base) = base_url.strip_suffix(suffix) {
            return base.trim_end_matches('/');
        }
    }
    base_url
}

fn base_has_path(base_url: &str) -> bool {
    let without_scheme = base_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(base_url);
    without_scheme
        .split_once('/')
        .map(|(_, path)| !path.trim_matches('/').is_empty())
        .unwrap_or(false)
}

fn call_compatible_with_fallback(request: &PromptRequest) -> Result<String, String> {
    let messages_error = match call_claude(request) {
        Ok(prompt) => return Ok(prompt),
        Err(err) => err,
    };

    if let Some(fallback_url) = &request.fallback_url {
        let fallback_request = PromptRequest {
            url: fallback_url.clone(),
            fallback_url: None,
            model: request.model.clone(),
            user_content: request.user_content.clone(),
            api_key: request.api_key.clone(),
            provider: request.provider.clone(),
        };
        call_openai_compatible(&fallback_request).map_err(|chat_error| {
            format!(
                "自定义兼容接口先尝试 Claude Messages 失败: {messages_error}; 再尝试 OpenAI Chat Completions 失败: {chat_error}"
            )
        })
    } else {
        Err(messages_error)
    }
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
        .config()
        .http_status_as_error(false)
        .build()
        .send_json(body)
        .map_err(|err| format!("提示词优化 API 调用失败: {err}"))?;
    let response_text = read_api_response_text(response, "提示词优化 API")?;
    let parsed: OpenAiChatResponse = parse_json_response(&response_text, "提示词优化 API")?;
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
        .config()
        .http_status_as_error(false)
        .build()
        .send_json(body)
        .map_err(|err| format!("Claude 提示词优化 API 调用失败: {err}"))?;
    let response_text = read_api_response_text(response, "Claude")?;
    let parsed: Value = parse_json_response(&response_text, "Claude")?;
    extract_claude_text(&parsed).ok_or_else(|| "Claude 没有返回提示词内容。".to_string())
}

fn read_api_response_text(
    mut response: ureq::http::Response<ureq::Body>,
    provider_label: &str,
) -> Result<ApiResponseText, String> {
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|err| format!("{provider_label} 响应读取失败: {err}"))?;

    if !status.is_success() {
        return Err(format!(
            "{provider_label} API 返回 HTTP {}，响应片段: {}",
            status.as_u16(),
            response_preview(&body)
        ));
    }

    Ok(ApiResponseText {
        status: status.as_u16(),
        content_type,
        body,
    })
}

fn parse_json_response<T: serde::de::DeserializeOwned>(
    response: &ApiResponseText,
    provider_label: &str,
) -> Result<T, String> {
    if !looks_like_json(&response.body) {
        return Err(format!(
            "{provider_label} 返回的不是 JSON (HTTP {}, Content-Type: {})，响应片段: {}",
            response.status,
            response.content_type,
            response_preview(&response.body)
        ));
    }

    serde_json::from_str(&response.body).map_err(|err| {
        format!(
            "{provider_label} 响应解析失败: {err}。响应片段: {}",
            response_preview(&response.body)
        )
    })
}

fn looks_like_json(body: &str) -> bool {
    matches!(body.trim_start().chars().next(), Some('{') | Some('['))
}

fn response_preview(body: &str) -> String {
    let compact = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return "<empty response body>".to_string();
    }

    let mut chars = compact.chars();
    let mut preview = chars.by_ref().take(400).collect::<String>();
    if chars.next().is_some() {
        preview.push_str("...");
    }
    preview
}

fn extract_claude_text(parsed: &Value) -> Option<String> {
    parsed
        .get("content")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .map(str::trim)
                .find(|value| !value.is_empty())
        })
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiProviderConfig, AppConfig};

    #[test]
    fn rejects_disabled_prompt_optimization_config() {
        let config = AppConfig::default();

        let error =
            build_prompt_request(&config, "local prompt").expect_err("disabled config should fail");

        assert!(error.contains("提示词优化 API 未启用"));
    }

    #[test]
    fn builds_compatible_prompt_request_with_messages_first() {
        let mut config = AppConfig::default();
        config.prompt_optimization = ApiProviderConfig {
            enabled: true,
            provider: "compatible".to_string(),
            base_url: "https://llm.example.com/v1".to_string(),
            model: "custom-model".to_string(),
            api_key: "sk-test".to_string(),
        };

        let request =
            build_prompt_request(&config, "一只猫，电影感").expect("request should build");

        assert_eq!(request.url, "https://llm.example.com/v1/messages");
        assert_eq!(
            request.fallback_url.as_deref(),
            Some("https://llm.example.com/v1/chat/completions")
        );
        assert_eq!(request.model, "custom-model");
        assert!(request.user_content.contains("一只猫"));
    }

    #[test]
    fn builds_compatible_prompt_request_from_gateway_root() {
        let mut config = AppConfig::default();
        config.prompt_optimization = ApiProviderConfig {
            enabled: true,
            provider: "compatible".to_string(),
            base_url: "https://llm.example.com".to_string(),
            model: "custom-model".to_string(),
            api_key: "sk-test".to_string(),
        };

        let request =
            build_prompt_request(&config, "一只猫，电影感").expect("request should build");

        assert_eq!(request.url, "https://llm.example.com/v1/messages");
        assert_eq!(
            request.fallback_url.as_deref(),
            Some("https://llm.example.com/v1/chat/completions")
        );
    }

    #[test]
    fn builds_compatible_prompt_request_from_full_chat_endpoint() {
        let mut config = AppConfig::default();
        config.prompt_optimization = ApiProviderConfig {
            enabled: true,
            provider: "compatible".to_string(),
            base_url: "https://llm.example.com/v1/chat/completions".to_string(),
            model: "custom-model".to_string(),
            api_key: "sk-test".to_string(),
        };

        let request =
            build_prompt_request(&config, "一只猫，电影感").expect("request should build");

        assert_eq!(request.url, "https://llm.example.com/v1/messages");
        assert_eq!(
            request.fallback_url.as_deref(),
            Some("https://llm.example.com/v1/chat/completions")
        );
    }

    #[test]
    fn does_not_duplicate_full_claude_messages_endpoint() {
        let mut config = AppConfig::default();
        config.prompt_optimization = ApiProviderConfig {
            enabled: true,
            provider: "claude".to_string(),
            base_url: "https://llm.example.com/v1/messages".to_string(),
            model: "claude-model".to_string(),
            api_key: "sk-test".to_string(),
        };

        let request =
            build_prompt_request(&config, "一只猫，电影感").expect("request should build");

        assert_eq!(request.url, "https://llm.example.com/v1/messages");
    }

    #[test]
    fn extracts_claude_text_after_thinking_blocks() {
        let response = json!({
            "content": [
                {
                    "type": "thinking",
                    "thinking": "internal reasoning"
                },
                {
                    "type": "text",
                    "text": "优化后的提示词"
                }
            ]
        });

        let text = extract_claude_text(&response).expect("text block should be extracted");

        assert_eq!(text, "优化后的提示词");
    }
}
