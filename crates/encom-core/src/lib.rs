//! Encom core daemon library.
//!
//! Public surface: [`Daemon`] which wires memory, IPC, the scheduler, and
//! model adapters together. The `encom-cli` binary spawns one of these.

use anyhow::{Context, Result};
use encom_ipc::{bind, write_frame, FrameReader, Frame, DEFAULT_BIND};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;

pub mod config;

pub use config::Config;

const SERVER_NAME: &str = "encom";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Daemon {
    pub config: Arc<Config>,
    pub state_dir: PathBuf,
    pub bind_addr: String,
}

impl Daemon {
    pub fn new(config: Config, state_dir: PathBuf) -> Self {
        Self {
            config: Arc::new(config),
            state_dir,
            bind_addr: DEFAULT_BIND.into(),
        }
    }

    pub fn with_bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    /// Run the daemon until Ctrl-C or fatal accept error.
    ///
    /// Phase 1: bind a TCP IPC listener on `bind_addr`, accept multiple
    /// concurrent connections, perform the Hello/HelloAck handshake, and
    /// echo Chat → ChatChunk(s) + ChatDone. Real model dispatch lands in
    /// Phase 2 — for now the model is a deterministic stub so we can verify
    /// the wire end-to-end.
    pub async fn run(self) -> Result<()> {
        tracing::info!(
            state_dir = %self.state_dir.display(),
            default_model = %self.config.model.default,
            bind = %self.bind_addr,
            version = SERVER_VERSION,
            "encom daemon starting"
        );
        std::fs::create_dir_all(&self.state_dir).ok();

        let listener = bind(&self.bind_addr).await?;

        let shutdown = tokio::signal::ctrl_c();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("shutdown signal received");
                    return Ok(());
                }
                accept = listener.accept() => {
                    let (stream, peer) = accept.context("accept")?;
                    let cfg = Arc::clone(&self.config);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, peer, cfg).await {
                            tracing::warn!(?peer, error = %e, "connection ended with error");
                        }
                    });
                }
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer: std::net::SocketAddr,
    _cfg: Arc<Config>,
) -> Result<()> {
    tracing::info!(?peer, "client connected");
    let (rd, mut wr) = stream.into_split();
    let mut reader = FrameReader::new(rd);

    let hello = reader.next().await?;
    match hello {
        Some(Frame::Hello { client, version }) => {
            tracing::info!(client = %client, client_version = %version, "handshake");
            write_frame(
                &mut wr,
                &Frame::HelloAck {
                    server: SERVER_NAME.into(),
                    version: SERVER_VERSION.into(),
                },
            )
            .await?;
        }
        other => {
            write_frame(
                &mut wr,
                &Frame::Error {
                    message: format!("expected hello, got {:?}", other),
                },
            )
            .await
            .ok();
            return Ok(());
        }
    }

    while let Some(frame) = reader.next().await? {
        match frame {
            Frame::Chat { text } => {
                tracing::info!(turn_chars = text.len(), "chat turn");
                // Phase 1 stub: split echo into 3 chunks so streaming is
                // exercised. Phase 2 swaps this for the real model adapter.
                let reply = format!("[encom:stub] you said: {text}");
                let third = reply.len().div_ceil(3);
                for chunk in reply.as_bytes().chunks(third) {
                    write_frame(
                        &mut wr,
                        &Frame::ChatChunk {
                            text: String::from_utf8_lossy(chunk).into_owned(),
                        },
                    )
                    .await?;
                }
                write_frame(
                    &mut wr,
                    &Frame::ChatDone {
                        input_tokens: text.len().min(u32::MAX as usize) as u32,
                        output_tokens: reply.len().min(u32::MAX as usize) as u32,
                    },
                )
                .await?;
            }
            Frame::Hello { .. } | Frame::HelloAck { .. } => {
                write_frame(
                    &mut wr,
                    &Frame::Error {
                        message: "unexpected handshake frame mid-session".into(),
                    },
                )
                .await
                .ok();
            }
            Frame::ChatChunk { .. } | Frame::ChatDone { .. } | Frame::Error { .. } => {
                // server-bound only frames; ignore client misuse silently
            }
        }
    }
    tracing::info!(?peer, "client disconnected");
    Ok(())
}
