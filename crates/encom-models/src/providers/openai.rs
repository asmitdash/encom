//! OpenAI Chat Completions adapter.

use crate::{CompletionRequest, CompletionResponse, Message, ModelAdapter, Usage};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct OpenAi {
    api_key: String,
    default_model: String,
    client: reqwest::Client,
}

impl OpenAi {
    pub fn new(api_key: String, default_model: String) -> Self {
        Self {
            api_key,
            default_model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    model: String,
    choices: Vec<Choice>,
    usage: Option<UsageRaw>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessageOwned,
}

#[derive(Deserialize)]
struct ChatMessageOwned {
    content: String,
}

#[derive(Deserialize)]
struct UsageRaw {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[async_trait]
impl ModelAdapter for OpenAi {
    fn id(&self) -> &str {
        "openai"
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let model = req.model.as_deref().unwrap_or(&self.default_model);
        let messages: Vec<ChatMessage> = req
            .messages
            .iter()
            .map(|m| match m {
                Message::System(c) => ChatMessage {
                    role: "system",
                    content: c,
                },
                Message::User(c) => ChatMessage {
                    role: "user",
                    content: c,
                },
                Message::Assistant(c) => ChatMessage {
                    role: "assistant",
                    content: c,
                },
            })
            .collect();
        let body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": req.max_tokens.unwrap_or(1024),
            "temperature": req.temperature.unwrap_or(0.7),
        });
        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<ChatResponse>()
            .await?;
        let choice = resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no choice"))?;
        Ok(CompletionResponse {
            text: choice.message.content,
            model: resp.model,
            usage: resp.usage.map(|u| Usage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
            }),
        })
    }
}
