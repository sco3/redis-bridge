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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::sync::Arc;
    use fred::mocks::{Echo, SimpleMap, Buffer, MockCommand};

    #[test]
    fn test_redis_error_display() {
        let err = RedisError::Subscription("timeout".to_string());
        assert!(err.to_string().contains("Failed to subscribe"));
        assert!(err.to_string().contains("timeout"));

        let err = RedisError::ParseError(serde_json::from_str::<()>("bad").unwrap_err());
        assert!(err.to_string().contains("Failed to parse message"));
    }

    #[test]
    fn test_redis_subscriber_new() {
        let config = Config::try_parse_from(["redis-bridge"]).unwrap();
        let subscriber = RedisSubscriber::new(config.clone());
        assert_eq!(subscriber.config.redis_url, config.redis_url);
        assert_eq!(subscriber.config.redis_channel, config.redis_channel);
    }

    #[tokio::test]
    async fn test_mock_echo_returns_args() {
        let config = RedisConfig {
            mocks: Some(Arc::new(Echo)),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // Echo returns the arguments back
        let result: Vec<RedisValue> = client
            .set("foo", "bar", None, None, false)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], RedisValue::String("foo".into()));
        assert_eq!(result[1], RedisValue::String("bar".into()));
    }

    #[tokio::test]
    async fn test_mock_simple_map_set_get() {
        let simple_map = Arc::new(SimpleMap::new());
        let config = RedisConfig {
            mocks: Some(simple_map.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // Test SET
        let _: () = client.set("foo", "bar", None, None, false).await.unwrap();

        // Test GET
        let val: String = client.get("foo").await.unwrap();
        assert_eq!(val, "bar");
    }

    #[tokio::test]
    async fn test_mock_simple_map_del() {
        let simple_map = Arc::new(SimpleMap::new());
        let config = RedisConfig {
            mocks: Some(simple_map.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // SET a value
        let _: () = client.set("key1", "value1", None, None, false).await.unwrap();

        // DEL the key
        let count: i64 = client.del("key1").await.unwrap();
        assert_eq!(count, 1);

        // GET should return null
        let val: Option<String> = client.get("key1").await.unwrap();
        assert!(val.is_none());
    }

    #[tokio::test]
    async fn test_mock_simple_map_multiple_keys() {
        let simple_map = Arc::new(SimpleMap::new());
        let config = RedisConfig {
            mocks: Some(simple_map.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // Set multiple keys
        let _: () = client.set("user:1", "Alice", None, None, false).await.unwrap();
        let _: () = client.set("user:2", "Bob", None, None, false).await.unwrap();

        // Get both back
        let user1: String = client.get("user:1").await.unwrap();
        let user2: String = client.get("user:2").await.unwrap();
        assert_eq!(user1, "Alice");
        assert_eq!(user2, "Bob");

        // Delete one
        let count: i64 = client.del("user:1").await.unwrap();
        assert_eq!(count, 1);

        let user1_after: Option<String> = client.get("user:1").await.unwrap();
        assert!(user1_after.is_none());

        let user2_still: String = client.get("user:2").await.unwrap();
        assert_eq!(user2_still, "Bob");
    }

    #[tokio::test]
    async fn test_mock_buffer_records_commands() {
        let buffer = Arc::new(Buffer::new());
        let config = RedisConfig {
            mocks: Some(buffer.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // All commands return QUEUED
        let _: String = client.set("foo", "bar", None, None, false).await.unwrap();
        let _: String = client.get("foo").await.unwrap();

        assert_eq!(buffer.len(), 2);

        let commands = buffer.take();
        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].cmd, "SET");
        assert_eq!(commands[1].cmd, "GET");
    }

    #[tokio::test]
    async fn test_mock_buffer_clear() {
        let buffer = Arc::new(Buffer::new());
        let config = RedisConfig {
            mocks: Some(buffer.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        let _: String = client.set("foo", "bar", None, None, false).await.unwrap();
        assert_eq!(buffer.len(), 1);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_simple_map_json_message() {
        let simple_map = Arc::new(SimpleMap::new());
        let config = RedisConfig {
            mocks: Some(simple_map.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // Store a JSON payload
        let json_value = serde_json::json!({
            "type": "tool_created",
            "name": "weather_api",
            "version": "1.0.0"
        });

        let _: () = client
            .set("notification:1", json_value.to_string(), None, None, false)
            .await
            .unwrap();

        let stored: String = client.get("notification:1").await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&stored).unwrap();
        assert_eq!(parsed["type"], "tool_created");
        assert_eq!(parsed["name"], "weather_api");
    }

    #[tokio::test]
    async fn test_mock_simple_map_clear() {
        let simple_map = Arc::new(SimpleMap::new());
        let config = RedisConfig {
            mocks: Some(simple_map.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        let _: () = client.set("key1", "val1", None, None, false).await.unwrap();
        let _: () = client.set("key2", "val2", None, None, false).await.unwrap();

        simple_map.clear();

        let val1: Option<String> = client.get("key1").await.unwrap();
        let val2: Option<String> = client.get("key2").await.unwrap();
        assert!(val1.is_none());
        assert!(val2.is_none());
    }

    #[tokio::test]
    async fn test_mock_pipeline_with_echo() {
        let config = RedisConfig {
            mocks: Some(Arc::new(Echo)),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        let pipeline = client.pipeline();
        pipeline.get::<(), _>("foo").await.unwrap();
        pipeline.get::<(), _>("bar").await.unwrap();

        let all: Vec<Vec<String>> = pipeline.all().await.unwrap();
        assert_eq!(all, vec![vec!["foo"], vec!["bar"]]);
    }

    #[tokio::test]
    async fn test_mock_subscribe_with_buffer() {
        let buffer = Arc::new(Buffer::new());
        let config = RedisConfig {
            mocks: Some(buffer.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // Subscribe should be recorded in buffer
        client.subscribe("test_channel").await.unwrap();

        let commands = buffer.take();
        assert!(!commands.is_empty());
        assert_eq!(commands[0].cmd, "SUBSCRIBE");
    }

    #[tokio::test]
    async fn test_mock_publish_with_buffer() {
        let buffer = Arc::new(Buffer::new());
        let config = RedisConfig {
            mocks: Some(buffer.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        // Buffer returns QUEUED, so we use String return type
        let _: String = client.publish("notifications", "hello").await.unwrap();

        let commands = buffer.take();
        assert!(!commands.is_empty());
        assert_eq!(commands[0].cmd, "PUBLISH");
    }

    /// Custom mock implementing a realistic tool-notification scenario
    #[derive(Debug)]
    struct ToolNotificationMock {
        publish_buffer: std::sync::Mutex<Vec<(String, String)>>,
    }

    impl ToolNotificationMock {
        fn new() -> Self {
            Self {
                publish_buffer: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn take_published(&self) -> Vec<(String, String)> {
            self.publish_buffer.lock().unwrap().drain(..).collect()
        }
    }

    impl fred::mocks::Mocks for ToolNotificationMock {
        fn process_command(&self, command: MockCommand) -> Result<RedisValue, fred::error::RedisError> {
            if &*command.cmd == "PUBLISH" {
                let channel = match command.args.first() {
                    Some(RedisValue::String(s)) => s.to_string(),
                    Some(RedisValue::Bytes(b)) => String::from_utf8_lossy(&b).to_string(),
                    _ => return Err(fred::error::RedisError::new(fred::error::RedisErrorKind::InvalidArgument, "Invalid channel")),
                };
                let message = match command.args.get(1) {
                    Some(RedisValue::String(s)) => s.to_string(),
                    Some(RedisValue::Bytes(b)) => String::from_utf8_lossy(&b).to_string(),
                    _ => return Err(fred::error::RedisError::new(fred::error::RedisErrorKind::InvalidArgument, "Invalid message")),
                };
                self.publish_buffer.lock().unwrap().push((channel, message));
                Ok(RedisValue::Integer(1))
            } else if &*command.cmd == "SUBSCRIBE" {
                Ok(RedisValue::Queued)
            } else {
                Err(fred::error::RedisError::new(fred::error::RedisErrorKind::Unknown, "Unimplemented."))
            }
        }
    }

    #[tokio::test]
    async fn test_mock_custom_tool_notification() {
        let mock = Arc::new(ToolNotificationMock::new());
        let config = RedisConfig {
            mocks: Some(mock.clone()),
            ..Default::default()
        };
        let client = RedisClient::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await.unwrap();

        let notification = serde_json::json!({
            "event_type": "tool_created",
            "tool": {
                "name": "get_weather",
                "url": "http://api.weather.com/v1",
                "description": "Fetch weather data",
            }
        });

        let count: i64 = client
            .publish("tool_notifications", notification.to_string())
            .await
            .unwrap();

        assert_eq!(count, 1);

        let published = mock.take_published();
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0, "tool_notifications");

        let parsed: serde_json::Value = serde_json::from_str(&published[0].1).unwrap();
        assert_eq!(parsed["event_type"], "tool_created");
        assert_eq!(parsed["tool"]["name"], "get_weather");
    }
}
