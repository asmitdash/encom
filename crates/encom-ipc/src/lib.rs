//! IPC protocol between the `encom` CLI and the daemon.
//!
//! Wire format: newline-delimited JSON. Each frame is one [`Frame`].
//! Phase 0 defines the shape only; transport (Unix socket vs Windows named
//! pipe) lands in Phase 1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Frame {
    Hello { client: String, version: String },
    Chat { text: String },
    ChatChunk { text: String },
    ChatDone { input_tokens: u32, output_tokens: u32 },
    Error { message: String },
}
