//! Model adapter trait and built-in providers.
//!
//! Adding a new provider is one file: implement [`ModelAdapter`] and register
//! it in [`registry::default_registry`].

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod providers;
pub mod registry;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "role", content = "content")]
pub enum Message {
    System(String),
    User(String),
    Assistant(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub model: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[async_trait]
pub trait ModelAdapter: Send + Sync {
    /// Provider id, e.g. "openai", "anthropic", "ollama".
    fn id(&self) -> &str;

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse>;
}
