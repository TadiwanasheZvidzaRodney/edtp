use clickhouse::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(clickhouse::Row, Serialize, Deserialize, Debug, Clone)]
pub struct AlertRow {
    pub event_id: String,
    pub timestamp: String,
    pub event_date: String, // format YYYY-MM-DD for partitioning
    pub endpoint_id: String,
    pub tenant_id: String,
    pub severity: i32,
    pub message: String,
    pub category: String,
}

pub struct ClickHouseStore {
    client: Client,
}

impl ClickHouseStore {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let ch_user = env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string());
        let ch_password = env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "edtp".to_string());

        let client = Client::default()
            .with_url(url)
            .with_database("default")
            .with_user(ch_user)
            .with_password(ch_password);
            
        // Initialize the analytical schema with ZSTD compression
        client.query("
            CREATE TABLE IF NOT EXISTS alerts (
                event_id String,
                timestamp DateTime64(9),
                event_date Date,
                endpoint_id String CODEC(ZSTD(3)),
                tenant_id String CODEC(ZSTD(3)),
                severity Int32,
                message String CODEC(ZSTD(3)),
                category String CODEC(ZSTD(3))
            )
            ENGINE = MergeTree()
            PARTITION BY (tenant_id, event_date)
            ORDER BY (timestamp, endpoint_id)
        ").execute().await?;

        Ok(Self { client })
    }

    pub async fn insert_batch(&self, alerts: Vec<AlertRow>) -> anyhow::Result<()> {
        let mut insert = self.client.insert("alerts")?;
        for alert in alerts {
            insert.write(&alert).await?;
        }
        insert.end().await?;
        Ok(())
    }
}
