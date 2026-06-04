//! Configuration loaded from `~/.config/encom/config.toml`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub skills: SkillsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub default: String,
    #[serde(flatten)]
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key_env: Option<String>,
    pub host: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    #[serde(default = "default_memory_backend")]
    pub backend: String,
    pub path: Option<String>,
}

fn default_memory_backend() -> String { "sqlite".into() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsConfig {
    #[serde(default)]
    pub dirs: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("reading config at {}", path.display()))?;
        let cfg: Config = toml::from_str(&raw).context("parsing config.toml")?;
        Ok(cfg)
    }

    pub fn default_with_anthropic() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key_env: Some("ANTHROPIC_API_KEY".into()),
                host: None,
                model: Some("claude-opus-4-7".into()),
            },
        );
        Config {
            model: ModelConfig { default: "anthropic".into(), providers },
            memory: MemoryConfig::default(),
            skills: SkillsConfig::default(),
        }
    }
}
