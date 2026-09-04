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
    let interval_secs = std::env::var("TELEMETRY_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);

    std::thread::spawn(move || {
        let mut process_id_counter = 1000;
        
        loop {
            // Simulate an event burst with suspicious command lines for testing
            for i in 0..5 {
                let (image, cmd) = if i == 0 {
                    ("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".to_string(), "powershell.exe -hidden -enc ...".to_string())
                } else {
                    ("C:\\Windows\\System32\\svchost.exe".to_string(), "svchost.exe -k LocalServiceNetworkRestricted".to_string())
                };

                let event = TelemetryEvent::ProcessCreate(ProcessCreateEvent {
                    timestamp: Utc::now(),
                    process_id: process_id_counter,
                    parent_process_id: 4, // System
                    image_file_name: image,
                    command_line: cmd,
                });

                // If the receiver is dropped, we safely terminate the ingest thread.
                if sender.send(event).is_err() {
                    return;
                }
                
                process_id_counter += 1;
            }
            
            std::thread::sleep(Duration::from_secs(interval_secs));
        }
    });

    Ok(())
}
