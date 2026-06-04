//! Encom core daemon library.
//!
//! Public surface: [`Daemon`] which wires memory, IPC, the scheduler, and
//! model adapters together. The `encom-cli` binary spawns one of these.

use anyhow::Result;
use std::path::PathBuf;

pub mod config;

pub use config::Config;

pub struct Daemon {
    pub config: Config,
    pub state_dir: PathBuf,
}

impl Daemon {
    pub fn new(config: Config, state_dir: PathBuf) -> Self {
        Self { config, state_dir }
    }

    /// Run the daemon until Ctrl-C.
    ///
    /// Phase 0 is a placeholder: it logs that it's alive and blocks on a
    /// signal. Phase 1 will spawn the IPC listener, scheduler, and skill
    /// supervisor here.
    pub async fn run(self) -> Result<()> {
        tracing::info!(
            state_dir = %self.state_dir.display(),
            default_model = %self.config.model.default,
            "encom daemon starting (phase 0 stub)"
        );
        tokio::signal::ctrl_c().await?;
        tracing::info!("shutdown");
        Ok(())
    }
}
