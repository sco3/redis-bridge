use fred::prelude::*;
use thiserror::Error;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Failed to connect to Redis: {0}")]
    Connection(#[from] fred::error::RedisError),
    #[error("Failed to subscribe to channel: {0}")]
    Subscription(String),
    #[error("Failed to parse message: {0}")]
    ParseError(#[from] serde_json::Error),
}

pub struct RedisSubscriber {
    config: Config,
}

impl RedisSubscriber {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the Redis subscription loop, invoking the handler for each message.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection fails or the subscription cannot be established.
    pub async fn run<F, T>(
        &self,
        mut handler: F,
    ) -> Result<(), RedisError>
    where
        F: FnMut(serde_json::Value) -> T + Send + Sync + 'static,
        T: std::future::Future<Output = ()> + Send,
    {
        info!("Connecting to Redis at {}", self.config.redis_url);

        let config = RedisConfig::from_url(&self.config.redis_url)?;
        let client = RedisClient::new(config, None, None, None);

        // Connect to Redis — store the handle so the connection task isn't dropped
        let _conn_handle = client.connect();
        client.wait_for_connect().await?;

        // Get the message stream before subscribing
        let mut message_stream = client.message_rx();

        info!("Subscribing to channel: {}", self.config.redis_channel);
        client.subscribe(&self.config.redis_channel).await?;

        info!("Successfully subscribed to Redis channel");

        while let Ok(message) = message_stream.recv().await {
            let channel = message.channel;
            info!("Received message on channel: {}", channel);

            let payload: String = match message.value.convert() {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to convert message to string: {}", e);
                    continue;
                }
            };

            let json_value: serde_json::Value = match serde_json::from_str(&payload) {
                Ok(v) => v,
                Err(e) => {
                    warn!("Failed to parse JSON message: {}. Raw: {}", e, payload);
                    continue;
                }
            };

            handler(json_value).await;
        }

        warn!("Redis subscription ended");
        Ok(())
    }
}
