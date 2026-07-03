//! IPC protocol between the `encom` CLI and the daemon.
//!
//! Wire format: newline-delimited JSON over a localhost TCP socket. Each
//! frame is one [`Frame`]. Cross-platform by design — POSIX gets a Unix
//! socket layer in a later phase; Phase 1 ships TCP loopback so the same
//! code path is exercised on Linux, macOS, and Windows.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};

/// Default loopback bind for the Phase-1 TCP transport.
pub const DEFAULT_BIND: &str = "127.0.0.1:8765";

/// One framed message on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Frame {
    /// Sent by the client immediately after connecting.
    Hello { client: String, version: String },
    /// Server's reply to `Hello`.
    HelloAck { server: String, version: String },
    /// User input — a complete chat turn.
    Chat { text: String },
    /// One streamed chunk of the assistant's response.
    ChatChunk { text: String },
    /// End-of-turn marker carrying token usage.
    ChatDone {
        input_tokens: u32,
        output_tokens: u32,
    },
    /// Recoverable, per-turn error. Connection stays open.
    Error { message: String },
}

/// Wrap one half of a TCP socket as a line-delimited reader of [`Frame`]s.
pub struct FrameReader {
    inner: BufReader<OwnedReadHalf>,
    buf: String,
}

impl FrameReader {
    pub fn new(half: OwnedReadHalf) -> Self {
        Self {
            inner: BufReader::new(half),
            buf: String::with_capacity(4096),
        }
    }

    /// Read the next frame. Returns `Ok(None)` on clean EOF.
    pub async fn next(&mut self) -> Result<Option<Frame>> {
        self.buf.clear();
        let n = self
            .inner
            .read_line(&mut self.buf)
            .await
            .context("reading frame")?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = self.buf.trim_end_matches(|c| c == '\n' || c == '\r');
        if trimmed.is_empty() {
            return Ok(None);
        }
        let frame = serde_json::from_str::<Frame>(trimmed)
            .with_context(|| format!("decoding frame: {trimmed}"))?;
        Ok(Some(frame))
    }
}

/// Encode a single frame as one NDJSON line on `writer`.
pub async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, frame: &Frame) -> Result<()> {
    let line = serde_json::to_string(frame).context("encoding frame")?;
    writer.write_all(line.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

/// Bind the IPC listener.
pub async fn bind(addr: &str) -> Result<TcpListener> {
    TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))
}

/// Connect to a running daemon.
pub async fn connect(addr: &str) -> Result<(FrameReader, OwnedWriteHalf)> {
    let stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("connecting to {addr}"))?;
    let (rd, wr) = stream.into_split();
    Ok((FrameReader::new(rd), wr))
}

/// Convenience: send `Hello`, expect `HelloAck`. Returns the ack info.
pub async fn handshake(
    rd: &mut FrameReader,
    wr: &mut OwnedWriteHalf,
    client: &str,
    version: &str,
) -> Result<(String, String)> {
    write_frame(
        wr,
        &Frame::Hello {
            client: client.into(),
            version: version.into(),
        },
    )
    .await?;
    match rd.next().await? {
        Some(Frame::HelloAck { server, version }) => Ok((server, version)),
        Some(Frame::Error { message }) => Err(anyhow!("daemon rejected handshake: {message}")),
        other => Err(anyhow!("expected hello_ack, got {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_roundtrip() {
        let cases = vec![
            Frame::Hello {
                client: "test".into(),
                version: "0.0.1".into(),
            },
            Frame::HelloAck {
                server: "encom".into(),
                version: "0.0.1".into(),
            },
            Frame::Chat {
                text: "hello".into(),
            },
            Frame::ChatChunk { text: "hi".into() },
            Frame::ChatDone {
                input_tokens: 1,
                output_tokens: 2,
            },
            Frame::Error {
                message: "oops".into(),
            },
        ];
        for f in cases {
            let s = serde_json::to_string(&f).unwrap();
            let parsed: Frame = serde_json::from_str(&s).unwrap();
            // Re-encode and compare; round-trip must be stable.
            let s2 = serde_json::to_string(&parsed).unwrap();
            assert_eq!(s, s2);
        }
    }
}
