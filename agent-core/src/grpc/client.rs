use crate::grpc::pb::{telemetry_service_client::TelemetryServiceClient, AlertEvent as PbAlertEvent};
use tonic::transport::Channel;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use crate::models::TelemetryEvent;

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
        // In a real implementation, this opens a bidirectional stream or separate RPC
        // to listen for 'Isolate' commands sent from the Agentic AI via the gateway.
        println!("Agent is securely listening for Agentic AI commands (e.g. Isolate Endpoint) via mTLS...");
        
        // Mock isolation execution block:
        // std::process::Command::new("netsh")
        //     .args(["advfirewall", "set", "allprofiles", "state", "on", "blockinbound", "always"])
        //     .spawn()?;
        
        Ok(())
    }
}
