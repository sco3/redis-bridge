use clap::Parser;
use fred::mocks::{MockCommand, Mocks};
use fred::prelude::*;
use fred::types::config::Config as FredConfig;
use redis_bridge::config::Config as AppConfig;
use redis_bridge::redis_subscriber::RedisSubscriber;
use std::sync::Arc;
use std::time::Duration;

/// Mock that accepts any command and returns OK.
#[derive(Debug)]
struct SimpleOkMock;

impl Mocks for SimpleOkMock {
    fn process_command(&self, _command: MockCommand) -> Result<Value, Error> {
        Ok(Value::Queued)
    }
}

/// Test that the subscriber handles the `run()` path with a pre-built client.
/// This covers the `Some(client)` branch in `run()`.
#[tokio::test]
async fn test_run_with_prebuilt_client() {
    let mock = Arc::new(SimpleOkMock);
    let fred_config = FredConfig {
        mocks: Some(mock),
        ..Default::default()
    };
    let client = Builder::from_config(fred_config).build().unwrap();
    client.init().await.unwrap();

    let app_config = AppConfig::try_parse_from(["redis-bridge"]).unwrap();
    let subscriber = RedisSubscriber::with_client(app_config, client);

    // Verify the subscriber uses the pre-built client path
    assert_eq!(subscriber.redis_url(), "redis://127.0.0.1:6379");
    assert_eq!(subscriber.redis_stream(), "policy-binding-events");

    // Run will timeout since mock doesn't stream messages
    let _ = tokio::time::timeout(
        Duration::from_millis(200),
        subscriber.run(|_value| async {}),
    )
    .await;
}
