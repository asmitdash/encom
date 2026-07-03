//! `encom` — the command-line front door.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use encom_core::{Config, Daemon};
use encom_ipc::{connect, handshake, write_frame, Frame, DEFAULT_BIND};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};

const CLIENT_NAME: &str = "encom-cli";
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "encom", version, about = "Encom agent runtime")]
struct Cli {
    /// Override the IPC bind/connect address (default 127.0.0.1:8765).
    #[arg(long, global = true)]
    addr: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the Encom daemon in the foreground.
    Daemon,
    /// Open an interactive chat with the running daemon. Type `:quit` to exit.
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
    let addr = cli.addr.as_deref().unwrap_or(DEFAULT_BIND).to_string();

    match cli.cmd {
        Cmd::Daemon => {
            let cfg = load_or_default_config()?;
            let state = state_dir()?;
            std::fs::create_dir_all(&state).ok();
            Daemon::new(cfg, state).with_bind_addr(addr).run().await
        }
        Cmd::Chat => run_chat(&addr).await,
        Cmd::Config => {
            let cfg = load_or_default_config()?;
            println!("{}", toml::to_string_pretty(&cfg)?);
            Ok(())
        }
    }
}

async fn run_chat(addr: &str) -> Result<()> {
    let (mut rd, mut wr) = connect(addr).await?;
    let (server, version) = handshake(&mut rd, &mut wr, CLIENT_NAME, CLIENT_VERSION).await?;
    eprintln!("Connected to {server} v{version}. Type :quit to exit.");

    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    loop {
        eprint!("> ");
        // Flush eprintln/print buffers so the prompt actually appears
        // before we block on stdin. Stderr is unbuffered on most setups
        // but be defensive.
        use std::io::Write;
        std::io::stderr().flush().ok();

        let Some(line) = lines.next_line().await? else {
            break; // EOF
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == ":quit" || trimmed == ":q" {
            break;
        }

        write_frame(
            &mut wr,
            &Frame::Chat {
                text: trimmed.into(),
            },
        )
        .await?;
        loop {
            match rd.next().await? {
                Some(Frame::ChatChunk { text }) => {
                    print!("{text}");
                    std::io::stdout().flush().ok();
                }
                Some(Frame::ChatDone {
                    input_tokens,
                    output_tokens,
                }) => {
                    println!();
                    eprintln!("[done · in={input_tokens} out={output_tokens}]");
                    break;
                }
                Some(Frame::Error { message }) => {
                    eprintln!("[error] {message}");
                    break;
                }
                Some(other) => {
                    eprintln!("[unexpected frame] {other:?}");
                    break;
                }
                None => return Err(anyhow::anyhow!("daemon closed connection")),
            }
        }
    }
    Ok(())
}

fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().context("no config dir")?.join("encom");
    Ok(dir.join("config.toml"))
}

fn state_dir() -> Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .context("no state dir")?
        .join("encom"))
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
