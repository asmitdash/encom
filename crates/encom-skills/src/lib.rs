//! Skill manifest + loader.
//!
//! A skill is a directory with `encom.toml` + an entry script. The host loads
//! the manifest, validates declared permissions, then runs the entry script
//! inside a V8 isolate (Phase 2). Phase 0 only parses manifests.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub permissions: Permissions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default)]
    pub fs: Vec<String>,
    #[serde(default)]
    pub secrets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub manifest: Manifest,
    pub root: PathBuf,
}

impl Skill {
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let manifest_path = dir.join("encom.toml");
        let raw = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?;
        let manifest: Manifest = toml::from_str(&raw).context("parsing manifest")?;
        Ok(Self { manifest, root: dir.to_path_buf() })
    }
}
