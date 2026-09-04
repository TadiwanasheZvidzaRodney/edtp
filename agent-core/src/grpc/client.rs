use crate::grpc::pb::{telemetry_service_client::TelemetryServiceClient, AlertEvent as PbAlertEvent};
use anyhow::Context;
use serde::Deserialize;
use std::env;
use std::process::Command;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use crate::models::TelemetryEvent;

#[derive(Debug, Deserialize)]
struct EndpointCommand {
    command_id: String,
    action: String,
    endpoint_id: String,
    tenant_id: String,
    reason: String,
    source_alert_id: String,
    issued_at: String,
}

pub struct GrpcAlertClient {
    tx: mpsc::Sender<PbAlertEvent>,
}

impl GrpcAlertClient {
    pub async fn connect(endpoint: String) -> anyhow::Result<Self> {
        let mut client = TelemetryServiceClient::connect(endpoint).await?;
        let (tx, rx) = mpsc::channel(1000);
        
        // Spawn a background task to stream alerts to the server
        tokio::spawn(async move {
            let stream = ReceiverStream::new(rx);
            if let Err(e) = client.stream_alerts(stream).await {
                eprintln!("gRPC Stream Error: {}", e);
            }
        });

        Ok(Self { tx })
    }

    pub async fn send_alert(&self, event: &TelemetryEvent, category: &str, message: &str) -> anyhow::Result<()> {
        let (event_id, timestamp, severity) = match event {
            TelemetryEvent::ProcessCreate(pc) => (
                format!("{}-{}", pc.process_id, pc.timestamp.timestamp_nanos_opt().unwrap_or(0)),
                pc.timestamp.to_rfc3339(),
                8, // High severity
            ),
            TelemetryEvent::ProcessTerminate(pt) => (
                format!("{}-{}", pt.process_id, pt.timestamp.timestamp_nanos_opt().unwrap_or(0)),
                pt.timestamp.to_rfc3339(),
                2,
            ),
        };

        let alert = PbAlertEvent {
            event_id,
            timestamp,
            endpoint_id: "EP-WIN-001".to_string(),
            tenant_id: "T-1000".to_string(),
            severity,
            message: message.to_string(),
            category: category.to_string(),
        };

        self.tx.send(alert).await?;
        Ok(())
    }

    pub async fn listen_for_commands(&self) -> anyhow::Result<()> {
        let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "127.0.0.1:4222".to_string());
        let command_subject = env::var("NATS_COMMAND_SUBJECT")
            .unwrap_or_else(|_| "telemetry.commands".to_string());
        let isolate_enabled = env::var("ENABLE_ENDPOINT_ISOLATION")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let nats_client = async_nats::connect(&nats_url)
            .await
            .with_context(|| format!("Failed to connect to NATS at {}", nats_url))?;
        let mut subscriber = nats_client
            .subscribe(command_subject.clone())
            .await
            .with_context(|| format!("Failed to subscribe to {}", command_subject))?;

        println!(
            "Agent command listener connected to NATS subject '{}' (isolation execution: {})",
            command_subject,
            if isolate_enabled { "enabled" } else { "dry-run" }
        );

        tokio::spawn(async move {
            let _keep_alive = nats_client;

            while let Some(msg) = subscriber.next().await {
                let parsed = serde_json::from_slice::<EndpointCommand>(&msg.payload);
                match parsed {
                    Ok(command) => {
                        println!(
                            "Received command {} for endpoint {} (tenant {}), source alert {} at {}: {}",
                            command.command_id,
                            command.endpoint_id,
                            command.tenant_id,
                            command.source_alert_id,
                            command.issued_at,
                            command.reason
                        );

                        if command.action.eq_ignore_ascii_case("isolate") {
                            if isolate_enabled {
                                let status = Command::new("netsh")
                                    .args(["advfirewall", "set", "allprofiles", "state", "on"])
                                    .status();

                                match status {
                                    Ok(exit) if exit.success() => {
                                        println!("Isolation action executed successfully via netsh");
                                    }
                                    Ok(exit) => {
                                        eprintln!("Isolation command failed with exit status: {}", exit);
                                    }
                                    Err(err) => {
                                        eprintln!("Failed to invoke isolation command: {}", err);
                                    }
                                }
                            } else {
                                println!(
                                    "Dry-run mode: set ENABLE_ENDPOINT_ISOLATION=true to execute host isolation"
                                );
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to parse command payload: {}", err);
                    }
                }
            }
        });

        Ok(())
    }
}
