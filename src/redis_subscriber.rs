use futures_util::StreamExt;
use thiserror::Error;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Failed to connect to Redis: {0}")]
    Connection(#[from] redis::RedisError),
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

        let client = redis::Client::open(self.config.redis_url.as_str())?;

        info!("Subscribing to channel: {}", self.config.redis_channel);

        #[allow(deprecated)]
        let conn = client
            .get_async_connection()
            .await
            .map_err(|e| RedisError::Subscription(e.to_string()))?;

        let mut pubsub = conn.into_pubsub();

        pubsub
            .subscribe(self.config.redis_channel.clone())
            .await
            .map_err(|e| RedisError::Subscription(e.to_string()))?;

        info!("Successfully subscribed to Redis channel");

        // Get the message stream
        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let channel = msg.get_channel_name().to_string();
            let payload: String = msg
                .get_payload()
                .map_err(|e| RedisError::Subscription(e.to_string()))?;

            info!("Received message on channel: {}", channel);

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
