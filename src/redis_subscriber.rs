use fred::prelude::*;
use thiserror::Error;
use tracing::{info, warn};

use crate::config::Config as AppConfig;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Failed to connect to Redis: {0}")]
    Connection(#[from] Error),
    #[error("Failed to subscribe to channel: {0}")]
    Subscription(String),
    #[error("Failed to parse message: {0}")]
    ParseError(#[from] serde_json::Error),
}

pub struct RedisSubscriber {
    config: AppConfig,
    client: Option<Client>,
}

impl RedisSubscriber {
    /// Create a subscriber that will build its own Redis client from config.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    /// Create a subscriber with a pre-built Redis client (for testing with mocks).
    #[must_use]
    pub fn with_client(config: AppConfig, client: Client) -> Self {
        Self {
            config,
            client: Some(client),
        }
    }

    /// Returns the Redis URL this subscriber is configured with.
    #[must_use]
    pub fn redis_url(&self) -> &str {
        &self.config.redis_url
    }

    /// Returns the Redis channel this subscriber subscribes to.
    #[must_use]
    pub fn redis_channel(&self) -> &str {
        &self.config.redis_channel
    }

    /// Run the Redis subscription loop, invoking the handler for each message.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection fails or the subscription cannot be established.
    pub async fn run<F, T>(&self, handler: F) -> Result<(), RedisError>
    where
        F: FnMut(serde_json::Value) -> T + Send + Sync + 'static,
        T: std::future::Future<Output = ()> + Send,
    {
        match &self.client {
            Some(client) => self.run_loop(client, handler).await,
            None => {
                info!("Connecting to Redis at {}", self.config.redis_url);
                let redis_config = fred::types::config::Config::from_url(&self.config.redis_url)?;
                let client = Builder::from_config(redis_config).build()?;
                client.init().await?;
                self.run_loop(&client, handler).await
            }
        }
    }

    async fn run_loop<F, T>(&self, client: &Client, mut handler: F) -> Result<(), RedisError>
    where
        F: FnMut(serde_json::Value) -> T + Send + Sync + 'static,
        T: std::future::Future<Output = ()> + Send,
    {
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
