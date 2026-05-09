#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptOptimizationProviderConfig {
    pub enabled: bool,
    pub provider: PromptOptimizationProvider,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptOptimizationProvider {
    LocalRules,
    OpenAi,
    Claude,
    Compatible,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationProviderConfig {
    pub enabled: bool,
    pub provider: ImageGenerationProvider,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageGenerationProvider {
    Disabled,
    GptImage,
    Compatible,
}
