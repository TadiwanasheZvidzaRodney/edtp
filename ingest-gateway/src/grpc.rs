pub mod pb {
    tonic::include_proto!("edtp.telemetry.v1");
}

use pb::telemetry_service_server::TelemetryService;
use pb::{AlertEvent, TelemetryResponse};
use tonic::{Request, Response, Status, Streaming};
use crate::nats_client::NatsPublisher;
use std::sync::Arc;
use serde::Serialize;

// We need a serializable version of the protobuf message for NATS
#[derive(Serialize)]
struct SerializableAlert {
    event_id: String,
    timestamp: String,
    endpoint_id: String,
    tenant_id: String,
    severity: i32,
    message: String,
    category: String,
}

pub struct TelemetryServerImpl {
    nats: Arc<NatsPublisher>,
}

impl TelemetryServerImpl {
    pub fn new(nats: Arc<NatsPublisher>) -> Self {
        Self { nats }
    }
}

#[tonic::async_trait]
impl TelemetryService for TelemetryServerImpl {
    async fn stream_alerts(
        &self,
        request: Request<Streaming<AlertEvent>>,
    ) -> Result<Response<TelemetryResponse>, Status> {
        let mut stream = request.into_inner();
        let mut received = 0;

        while let Some(alert_res) = stream.message().await? {
            let alert = alert_res;
            
            let serializable = SerializableAlert {
                event_id: alert.event_id,
                timestamp: alert.timestamp,
                endpoint_id: alert.endpoint_id,
                tenant_id: alert.tenant_id,
                severity: alert.severity,
                message: alert.message,
                category: alert.category,
            };

            if let Err(e) = self.nats.publish(&serializable).await {
                eprintln!("Failed to publish alert to NATS: {}", e);
                return Err(Status::internal("Message Queue Error"));
            }
            received += 1;
        }

        Ok(Response::new(TelemetryResponse {
            success: true,
            message: format!("Successfully buffered {} alerts", received),
        }))
    }
}
