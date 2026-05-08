use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct UsageParams {
    pub r#type: String,            // "book" | "code" | "agents" | "mixed"
    #[serde(default)]
    pub pages_per_chunk: Option<u32>,
    #[serde(default)]
    pub loc_per_chunk: Option<u32>,
    #[serde(default)]
    pub context_tokens: Option<u32>,
    #[serde(default)]
    pub max_tokens_per_call: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UsageSpec {
    pub i18n_key: String,
    pub weights: HashMap<String, f32>,
    pub params: UsageParams,
}

#[derive(Debug, Deserialize)]
pub struct UsagesConfig {
    pub usages: HashMap<String, UsageSpec>,
}