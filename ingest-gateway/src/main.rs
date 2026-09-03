mod clickhouse;
mod grpc;
mod nats_client;

use std::sync::Arc;
use tokio_stream::StreamExt;
use tonic::transport::Server;
use crate::grpc::pb::telemetry_service_server::TelemetryServiceServer;
use crate::grpc::TelemetryServerImpl;
use crate::nats_client::NatsPublisher;
use crate::clickhouse::{ClickHouseStore, AlertRow};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Ingestion Gateway (Phase 3)...");

    let nats_url = "127.0.0.1:4222";
    let clickhouse_url = "http://127.0.0.1:8123";

    // 1. Initialize ClickHouse (Ensures analytical schema exists)
    let ch_store = Arc::new(ClickHouseStore::new(clickhouse_url).await
        .unwrap_or_else(|e| {
            eprintln!("Warning: ClickHouse not reachable: {}. Running without DB for dev.", e);
            // We use a dummy for dev if CH is offline, but in a real system we'd crash.
            // For Phase 3 completeness, let's assume it might not be running locally.
            panic!("Critical: ClickHouse initialization failed");
        }));

    // 2. Initialize NATS Publisher for the gRPC Server
    let nats_publisher = Arc::new(NatsPublisher::new(nats_url, "telemetry.alerts").await
        .unwrap_or_else(|e| {
            panic!("Critical: NATS initialization failed: {}", e);
        }));

    // 3. Start NATS to ClickHouse Worker (The Consumer)
    let nats_client = async_nats::connect(nats_url).await?;
    let mut subscriber = nats_client.subscribe("telemetry.alerts").await?;
    
    let ch_store_worker = ch_store.clone();
    tokio::spawn(async move {
        println!("ClickHouse Worker: Listening for alerts on NATS...");
        let mut batch = Vec::new();
        
        while let Some(msg) = subscriber.next().await {
            if let Ok(alert) = serde_json::from_slice::<AlertRow>(&msg.payload) {
                batch.push(alert);
                
                // Flush batch when it hits 1000 items (or use a timer in prod)
                if batch.len() >= 1000 {
                    if let Err(e) = ch_store_worker.insert_batch(batch.clone()).await {
                        eprintln!("Failed to flush batch to ClickHouse: {}", e);
                    } else {
                        println!("Flushed 1000 alerts to ClickHouse.");
                    }
                    batch.clear();
                }
            }
        }
    });

    // 4. Start gRPC API Gateway
    let addr = "0.0.0.0:50051".parse()?;
    let server_impl = TelemetryServerImpl::new(nats_publisher);

    println!("gRPC Telemetry Gateway listening on {}", addr);
    Server::builder()
        .add_service(TelemetryServiceServer::new(server_impl))
        .serve(addr)
        .await?;

    Ok(())
}
