use async_nats::Client;
use serde::Serialize;
use anyhow::Context;

pub struct NatsPublisher {
    client: Client,
    topic: String,
}

impl NatsPublisher {
    pub async fn new(url: &str, topic: &str) -> anyhow::Result<Self> {
        let client = async_nats::connect(url).await
            .context("Failed to connect to NATS")?;
            
        // In a full deployment, we would also initialize the JetStream Context
        // and ensure the Stream exists here. For brevity, we assume auto-provisioning
        // or a pre-configured JetStream.
        
        Ok(Self {
            client,
            topic: topic.to_string(),
        })
    }

    pub async fn publish<T: Serialize>(&self, payload: &T) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(payload)?;
        self.client.publish(self.topic.clone(), bytes.into()).await?;
        Ok(())
    }
}
