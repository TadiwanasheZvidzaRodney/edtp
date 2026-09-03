use rusqlite::{params, Connection, Result};
use thiserror::Error;
use std::path::Path;
use chrono::{Duration, Utc};
use crate::models::TelemetryEvent;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub struct RingBuffer {
    conn: Connection,
}

impl RingBuffer {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, StorageError> {
        let conn = Connection::open(db_path)?;
        
        // Optimize for high-throughput write
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS telemetry_events (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                 event_type TEXT NOT NULL,
                 payload TEXT NOT NULL
             );"
        )?;

        Ok(Self { conn })
    }

    pub fn insert_event(&self, event: &TelemetryEvent) -> Result<(), StorageError> {
        let event_type = match event {
            TelemetryEvent::ProcessCreate(_) => "ProcessCreate",
            TelemetryEvent::ProcessTerminate(_) => "ProcessTerminate",
        };
        let payload = serde_json::to_string(event)?;

        self.conn.execute(
            "INSERT INTO telemetry_events (event_type, payload) VALUES (?1, ?2)",
            params![event_type, payload],
        )?;

        Ok(())
    }

    pub fn enforce_retention(&self, days: i64) -> Result<usize, StorageError> {
        let cutoff = Utc::now() - Duration::days(days);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

        let deleted = self.conn.execute(
            "DELETE FROM telemetry_events WHERE timestamp < ?1",
            params![cutoff_str],
        )?;

        Ok(deleted)
    }
}
