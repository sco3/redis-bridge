use clap::Parser;
use fred::mocks::{MockCommand, Mocks};
use fred::prelude::*;
use redis_bridge::config::Config;
use redis_bridge::redis_subscriber::RedisSubscriber;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Mock that publishes a message immediately after subscribe,
/// then closes the stream after a short delay.
#[derive(Debug)]
struct AutoPublishMock {
    message_count: AtomicUsize,
}

impl AutoPublishMock {
    fn new() -> Self {
        Self {
            message_count: AtomicUsize::new(0),
        }
    }

    fn count(&self) -> usize {
        self.message_count.load(Ordering::SeqCst)
    }
}

impl Mocks for AutoPublishMock {
    fn process_command(&self, command: MockCommand) -> Result<RedisValue, fred::error::RedisError> {
        match &*command.cmd {
            "SUBSCRIBE" => {
                self.message_count.fetch_add(1, Ordering::SeqCst);
                // Return a successful subscribe response
                Ok(RedisValue::Queued)
            }
            "PUBLISH" => {
                self.message_count.fetch_add(1, Ordering::SeqCst);
                Ok(RedisValue::Integer(1))
            }
            _ => Ok(RedisValue::Queued),
        }
    }
}

#[tokio::test]
async fn test_redis_subscriber_with_custom_client() {
    // Verify that with_client constructor works and the subscriber
    // correctly calls through to the mock client
    let mock = Arc::new(AutoPublishMock::new());
    let config = RedisConfig {
        mocks: Some(mock.clone()),
        ..Default::default()
    };
    let client = RedisClient::new(config, None, None, None);
    client.connect();
    client.wait_for_connect().await.unwrap();

    let app_config = Config::try_parse_from(["redis-bridge"]).unwrap();
    let subscriber = RedisSubscriber::with_client(app_config, client);

    // Run with timeout — the mock doesn't stream messages,
    // so we expect a timeout
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        subscriber.run(|_| async {}),
    )
    .await;

    // Timeout is expected (no messages delivered through stream in mock mode)
    assert!(result.is_err());

    // Verify subscribe was called
    assert!(mock.count() > 0);
}
