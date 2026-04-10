use fred::prelude::*;
use fred::types::streams::XReadResponse;
use thiserror::Error;
use tracing::{error, info, warn};

use crate::config::Config as AppConfig;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Failed to connect to Redis: {0}")]
    Connection(#[from] Error),
    #[error("Failed to create consumer group: {0}")]
    ConsumerGroupCreation(String),
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

    /// Returns the Redis stream key this subscriber reads from.
    #[must_use]
    pub fn redis_stream(&self) -> &str {
        &self.config.redis_stream
    }

    /// Run the Redis stream reading loop, invoking the handler for each message.
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis connection fails or the consumer group cannot be established.
    pub async fn run<F, T>(&self, handler: F) -> Result<(), RedisError>
    where
        F: FnMut(serde_json::Value) -> T + Send + Sync + 'static,
        T: std::future::Future<Output = ()> + Send,
    {
        if let Some(client) = &self.client {
            self.run_loop(client, handler).await
        } else {
            info!("Connecting to Redis at {}", self.config.redis_url);
            let redis_config = fred::types::config::Config::from_url(&self.config.redis_url)?;
            let client = Builder::from_config(redis_config).build()?;
            client.init().await?;
            self.run_loop(&client, handler).await
        }
    }

    /// Ensure the consumer group exists, creating it if necessary.
    async fn ensure_consumer_group(&self, client: &Client) -> Result<(), RedisError> {
        info!(
            "Ensuring consumer group '{}' exists for stream '{}'",
            self.config.redis_stream_group, self.config.redis_stream
        );

        // Try to create the consumer group with ID "0" (read from beginning)
        // mkstream=true will create the stream if it doesn't exist
        let result: Result<String, _> = client
            .xgroup_create(
                &self.config.redis_stream,
                &self.config.redis_stream_group,
                "0",
                true,
            )
            .await;

        match result {
            Ok(_) => {
                info!(
                    "Created consumer group '{}' for stream '{}'",
                    self.config.redis_stream_group, self.config.redis_stream
                );
            }
            Err(e) => {
                // If the group already exists, that's fine - continue
                if e.to_string().contains("BUSYGROUP") {
                    info!(
                        "Consumer group '{}' already exists for stream '{}'",
                        self.config.redis_stream_group, self.config.redis_stream
                    );
                } else {
                    return Err(RedisError::ConsumerGroupCreation(e.to_string()));
                }
            }
        }

        Ok(())
    }

    async fn run_loop<F, T>(&self, client: &Client, mut handler: F) -> Result<(), RedisError>
    where
        F: FnMut(serde_json::Value) -> T + Send + Sync + 'static,
        T: std::future::Future<Output = ()> + Send,
    {
        // Ensure the consumer group exists before reading
        self.ensure_consumer_group(client).await?;

        info!(
            "Reading from stream: {}, group: {}, consumer: {}",
            self.config.redis_stream,
            self.config.redis_stream_group,
            self.config.redis_stream_consumer
        );

        // Use ">" to read only new messages that were never delivered to this consumer group
        let last_id = ">";

        loop {
            info!(
                "Waiting for messages on stream '{}' from ID '{}'",
                self.config.redis_stream, last_id
            );

            // Block for up to 5 seconds waiting for messages
            let result: Result<XReadResponse<String, String, String, String>, _> = client
                .xreadgroup_map(
                    &self.config.redis_stream_group,
                    &self.config.redis_stream_consumer,
                    None,       // count - no limit
                    Some(5000), // block for 5 seconds (in milliseconds)
                    false,      // noack - false means we need to acknowledge
                    vec![self.config.redis_stream.as_str()],
                    vec![last_id],
                )
                .await;

            match result {
                Ok(response) => {
                    if response.is_empty() {
                        // No messages available (timeout), continue loop
                        info!("No messages available, waiting...");
                        continue;
                    }

                    for (_stream_key, entries) in response {
                        for (message_id, fields) in entries {
                            info!("Received message with ID: {}", message_id);

                            // Extract the payload from the stream entry
                            let payload = if let Some(p) = fields.get("payload") {
                                match serde_json::from_str(p) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("Failed to parse payload as JSON: {}. Raw: {}", e, p);
                                        continue;
                                    }
                                }
                            } else {
                                warn!("Stream entry has no 'payload' field");
                                continue;
                            };

                            // Process the message
                            handler(payload).await;

                            // Acknowledge the message
                            let msg_id: &str = &message_id;
                            let ack_result: Result<usize, _> = client
                                .xack(
                                    &self.config.redis_stream,
                                    &self.config.redis_stream_group,
                                    vec![msg_id],
                                )
                                .await;

                            match ack_result {
                                Ok(acked_count) => {
                                    info!(
                                        "Acknowledged message {}: {} acknowledged",
                                        message_id, acked_count
                                    );
                                }
                                Err(e) => {
                                    warn!("Failed to acknowledge message {}: {}", message_id, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stream: {}", e);
                    // Continue loop to retry
                }
            }
        }
    }
}
