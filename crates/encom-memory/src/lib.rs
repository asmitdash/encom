//! Persistent memory for Encom agents.
//!
//! Phase 0: SQLite-backed key/value + append-only log of `Memory` records,
//! addressed by namespace + key. Phase 1 adds a vector index for similarity
//! recall.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub namespace: String,
    pub key: String,
    pub value: String,
    pub created_at: i64,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).ok(); }
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn put(&self, namespace: &str, key: &str, value: &str) -> Result<Memory> {
        let id = Uuid::new_v4().to_string();
        let created_at = chrono_now_seconds();
        self.conn.execute(
            "INSERT INTO memories (id, namespace, key, value, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, namespace, key, value, created_at],
        )?;
        Ok(Memory { id, namespace: namespace.into(), key: key.into(), value: value.into(), created_at })
    }

    pub fn get_latest(&self, namespace: &str, key: &str) -> Result<Option<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, namespace, key, value, created_at FROM memories
             WHERE namespace = ?1 AND key = ?2
             ORDER BY created_at DESC LIMIT 1",
        )?;
        let row = stmt
            .query_row(params![namespace, key], |r| {
                Ok(Memory {
                    id: r.get(0)?, namespace: r.get(1)?, key: r.get(2)?,
                    value: r.get(3)?, created_at: r.get(4)?,
                })
            })
            .ok();
        Ok(row)
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS memories (
    id          TEXT PRIMARY KEY,
    namespace   TEXT NOT NULL,
    key         TEXT NOT NULL,
    value       TEXT NOT NULL,
    created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_memories_ns_key ON memories(namespace, key);
"#;

// std::time::SystemTime gives us seconds since epoch without a chrono dep.
fn chrono_now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
