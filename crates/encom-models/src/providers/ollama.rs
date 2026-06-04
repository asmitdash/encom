//! Local Ollama adapter — talks to a local Ollama server (default :11434).

use crate::{CompletionRequest, CompletionResponse, Message, ModelAdapter};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct Ollama {
    host: String,
    default_model: String,
    client: reqwest::Client,
}

impl Ollama {
    pub fn new(host: String, default_model: String) -> Self {
        Self { host, default_model, client: reqwest::Client::new() }
    }
}

#[derive(Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OllamaResponse {
    model: String,
    message: OllamaResponseMessage,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[async_trait]
impl ModelAdapter for Ollama {
    fn id(&self) -> &str { "ollama" }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let model = req.model.as_deref().unwrap_or(&self.default_model);
        let messages: Vec<OllamaMessage> = req.messages.iter().map(|m| match m {
            Message::System(c)    => OllamaMessage { role: "system",    content: c },
            Message::User(c)      => OllamaMessage { role: "user",      content: c },
            Message::Assistant(c) => OllamaMessage { role: "assistant", content: c },
        }).collect();
        let body = json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": req.temperature.unwrap_or(0.7),
                "num_predict": req.max_tokens.unwrap_or(1024),
            },
        });
        let resp = self.client
            .post(format!("{}/api/chat", self.host.trim_end_matches('/')))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<OllamaResponse>()
            .await?;
        Ok(CompletionResponse { text: resp.message.content, model: resp.model, usage: None })
    }
}
