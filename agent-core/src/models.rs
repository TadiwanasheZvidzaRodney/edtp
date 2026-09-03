use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TelemetryEvent {
    ProcessCreate(ProcessCreateEvent),
    ProcessTerminate(ProcessTerminateEvent),
    // Extensible for future events
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessCreateEvent {
    pub timestamp: DateTime<Utc>,
    pub process_id: u32,
    pub parent_process_id: u32,
    pub image_file_name: String,
    pub command_line: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessTerminateEvent {
    pub timestamp: DateTime<Utc>,
    pub process_id: u32,
    pub exit_status: u32,
}
