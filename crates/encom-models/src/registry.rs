//! Adapter registry — looks up a `ModelAdapter` by provider id.

use crate::providers::{anthropic, ollama, openai};
use crate::ModelAdapter;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Registry {
    adapters: HashMap<String, Arc<dyn ModelAdapter>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn insert(&mut self, adapter: Arc<dyn ModelAdapter>) {
        self.adapters.insert(adapter.id().to_string(), adapter);
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn ModelAdapter>> {
        self.adapters.get(id).cloned()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Default registry with built-in providers wired up from environment variables.
///
/// Phase 1 will replace env-only wiring with config-driven wiring.
pub fn default_registry() -> Registry {
    let mut reg = Registry::new();
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        reg.insert(Arc::new(openai::OpenAi::new(key, "gpt-5".into())));
    }
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        reg.insert(Arc::new(anthropic::Anthropic::new(
            key,
            "claude-opus-4-7".into(),
        )));
    }
    let ollama_host =
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".into());
    reg.insert(Arc::new(ollama::Ollama::new(
        ollama_host,
        "llama3.3".into(),
    )));
    reg
}
