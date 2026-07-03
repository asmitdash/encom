//! Anthropic Messages API adapter.

use crate::{CompletionRequest, CompletionResponse, Message, ModelAdapter, Usage};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct Anthropic {
    api_key: String,
    default_model: String,
    client: reqwest::Client,
}

impl Anthropic {
    pub fn new(api_key: String, default_model: String) -> Self {
        Self {
            api_key,
            default_model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct AnthroMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthroResponse {
    model: String,
    content: Vec<ContentBlock>,
    usage: Option<UsageRaw>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct UsageRaw {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl ModelAdapter for Anthropic {
    fn id(&self) -> &str {
        "anthropic"
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let model = req.model.as_deref().unwrap_or(&self.default_model);
        // Anthropic separates system prompt from messages.
        let mut system = String::new();
        let mut msgs: Vec<AnthroMessage> = Vec::new();
        for m in &req.messages {
            match m {
                Message::System(c) => {
                    if !system.is_empty() {
                        system.push('\n');
                    }
                    system.push_str(c);
                }
                Message::User(c) => msgs.push(AnthroMessage {
                    role: "user",
                    content: c,
                }),
                Message::Assistant(c) => msgs.push(AnthroMessage {
                    role: "assistant",
                    content: c,
                }),
            }
        }
        let body = json!({
            "model": model,
            "system": system,
            "messages": msgs,
            "max_tokens": req.max_tokens.unwrap_or(1024),
            "temperature": req.temperature.unwrap_or(0.7),
        });
        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<AnthroResponse>()
            .await?;
        let text = resp
            .content
            .into_iter()
            .filter_map(|b| b.text)
            .next()
            .ok_or_else(|| anyhow!("no text block in response"))?;
        Ok(CompletionResponse {
            text,
            model: resp.model,
            usage: resp.usage.map(|u| Usage {
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
            }),
        })
    }
}
