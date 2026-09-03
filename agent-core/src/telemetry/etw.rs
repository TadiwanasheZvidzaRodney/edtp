use crate::models::{ProcessCreateEvent, TelemetryEvent};
use chrono::Utc;
use crossbeam::channel::Sender;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EtwError {
    #[error("Failed to start ETW session: {0}")]
    SessionError(String),
}

/// Starts the ETW trace session. 
/// In a production environment, this interfaces with StartTraceW and ProcessTrace.
/// For this initial development phase, it produces an isolated synthetic event stream 
/// to validate the high-throughput lock-free ingestion and SQLite rolling buffer pipeline 
/// without requiring Kernel Driver/Administrator privileges.
pub fn start_trace_session(sender: Sender<TelemetryEvent>) -> Result<(), EtwError> {
    std::thread::spawn(move || {
        let mut process_id_counter = 1000;
        
        loop {
            // Simulate a high-throughput event burst
            for _ in 0..5 {
                let event = TelemetryEvent::ProcessCreate(ProcessCreateEvent {
                    timestamp: Utc::now(),
                    process_id: process_id_counter,
                    parent_process_id: 4, // System
                    image_file_name: "C:\\Windows\\System32\\svchost.exe".to_string(),
                    command_line: "svchost.exe -k LocalServiceNetworkRestricted".to_string(),
                });

                // If the receiver is dropped, we safely terminate the ingest thread.
                if sender.send(event).is_err() {
                    return;
                }
                
                process_id_counter += 1;
            }
            
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    Ok(())
}
