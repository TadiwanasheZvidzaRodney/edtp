mod models;
mod storage;
mod telemetry;
mod filtering;
mod grpc;

use anyhow::{Context, Result};
use crossbeam::channel;
use storage::ring_buffer::RingBuffer;
use std::path::PathBuf;
use filtering::rule_engine::RuleEngine;
use filtering::anomaly_engine::AnomalyEngine;
use grpc::client::GrpcAlertClient;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing Edge-AI Endpoint Agent (Phase 2)...");

    // Initialize Local Storage
    let db_path = PathBuf::from("telemetry_buffer.db");
    let ring_buffer = RingBuffer::new(&db_path)
        .context("Failed to initialize SQLite Ring Buffer")?;
    
    // Initialize Filtering & ML Engines
    let rule_engine = RuleEngine::new().context("Failed to load YAML rules")?;
    let anomaly_engine = AnomalyEngine::new(0.85); // Threshold

    // Connect to central SOC pipeline (NATS / gRPC gateway)
    // Note: Use a dummy port for now. In production, this points to the Redpanda/NATS gateway.
    let grpc_client = GrpcAlertClient::connect("http://127.0.0.1:50051".to_string()).await
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to connect to gRPC pipeline: {}. Continuing offline.", e);
            // In a real app we might retry or fail fast depending on exact requirements.
            // But for Phase 2 dev, we allow offline mode to test the local engines.
            // Let's panic to adhere to "Fail Fast & Explicit Handling"
            panic!("Critical: Cannot start without central connection: {}", e);
        });

    grpc_client
        .listen_for_commands()
        .await
        .context("Failed to start command listener")?;

    // Setup Lock-Free Channel
    let (tx, rx) = channel::bounded(10_000);

    // Spawn Processing Worker Thread
    let _worker = tokio::task::spawn_blocking(move || {
        // We use block_on to run async grpc send from the blocking thread
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        while let Ok(event) = rx.recv() {
            // 1. Raw Logging to SQLite
            let _ = ring_buffer.insert_event(&event);
            
            // 2. Rule Engine Evaluation
            if let Some(rule) = rule_engine.evaluate(&event) {
                println!("🚨 RULE TRIGGERED: {} - {}", rule.name, rule.description);
                let _ = rt.block_on(grpc_client.send_alert(&event, "Rule-Match", &rule.name));
                continue; // Skip anomaly engine if rule matches
            }

            // 3. ML Anomaly Engine Evaluation
            if anomaly_engine.is_anomalous(&event) {
                println!("🧠 ANOMALY DETECTED (Score: {:.2})", anomaly_engine.score_event(&event));
                let _ = rt.block_on(grpc_client.send_alert(&event, "Anomaly", "Behavioral deviation"));
            }
        }
    });

    println!("Starting telemetry ingest stream...");
    telemetry::etw::start_trace_session(tx)
        .context("Failed to start ETW trace session")?;

    println!("Agent is running with Phase 2 Filtering & ML active.");
    tokio::signal::ctrl_c().await.context("Failed to listen for ctrl+c")?;
    println!("Shutting down...");
    
    Ok(())
}
