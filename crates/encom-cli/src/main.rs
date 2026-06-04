//! `encom` — the command-line front door.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use encom_core::{Config, Daemon};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "encom", version, about = "Encom agent runtime")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the Encom daemon in the foreground.
    Daemon,
    /// Open an interactive chat with the running daemon.
    Chat,
    /// Print the resolved configuration.
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Daemon => {
            let cfg = load_or_default_config()?;
            let state = state_dir()?;
            std::fs::create_dir_all(&state).ok();
            Daemon::new(cfg, state).run().await
        }
        Cmd::Chat => {
            anyhow::bail!("chat is not implemented yet (phase 1)");
        }
        Cmd::Config => {
            let cfg = load_or_default_config()?;
            println!("{}", toml::to_string_pretty(&cfg)?);
            Ok(())
        }
    }
}

fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().context("no config dir")?.join("encom");
    Ok(dir.join("config.toml"))
}

fn state_dir() -> Result<PathBuf> {
    Ok(dirs::data_local_dir().context("no state dir")?.join("encom"))
}

fn load_or_default_config() -> Result<Config> {
    let path = config_path()?;
    if path.exists() {
        Config::load(&path)
    } else {
        tracing::warn!(path = %path.display(), "no config file, using anthropic-default");
        Ok(Config::default_with_anthropic())
    }
}
