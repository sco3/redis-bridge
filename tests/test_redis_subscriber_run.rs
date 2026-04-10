//! Integration test for `RedisSubscriber::run` against a real Redis instance.
//!
//! This requires a running Redis at `redis://127.0.0.1:6379`.
//! The test adds a message to the stream and verifies the handler receives it.

use clap::Parser;
use fred::prelude::*;
use redis_bridge::config::Config as AppConfig;
use redis_bridge::redis_subscriber::RedisSubscriber;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Skip if Redis is not available (e.g. in CI without services).
fn redis_available() -> bool {
    std::net::TcpStream::connect(("127.0.0.1", 6379)).is_ok()
}

/// Test that the subscriber's `run()` method:
/// 1. Connects and creates consumer group
/// 2. Receives a message added to the stream
/// 3. Passes the parsed JSON to the handler
#[tokio::test]
async fn test_redis_subscriber_receives_stream_message() {
    if !redis_available() {
        eprintln!("Skipping: Redis not available at 127.0.0.1:6379");
        return;
    }

    // Create a client to add messages to the stream
    let redis_cfg = fred::types::config::Config::from_url("redis://127.0.0.1:6379").unwrap();
    let publisher = Builder::from_config(redis_cfg).build().unwrap();
    publisher.init().await.unwrap();

    // Clean up any existing consumer group from previous test runs
    let _: String = publisher
        .xgroup_delconsumer("test_stream_integration", "test_group", "test_consumer")
        .await
        .unwrap_or(String::new());
    let _: String = publisher
        .xgroup_destroy("test_stream_integration", "test_group")
        .await
        .unwrap_or(String::new());
    let _ = publisher.del::<(), _>(&["test_stream_integration"]).await;

    // Create the subscriber with its own connection
    let app_cfg = AppConfig::try_parse_from([
        "redis-bridge",
        "--redis-stream",
        "test_stream_integration",
        "--redis-stream-group",
        "test_group",
        "--redis-stream-consumer",
        "test_consumer_integration",
    ])
    .unwrap();

    let redis_cfg = fred::types::config::Config::from_url("redis://127.0.0.1:6379").unwrap();
    let client = Builder::from_config(redis_cfg).build().unwrap();
    client.init().await.unwrap();

    let subscriber = RedisSubscriber::with_client(app_cfg, client.clone());

    // Shared state to verify handler was called
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();

    // Spawn the subscriber loop
    let received_clone2 = received.clone();
    let sub_handle = tokio::spawn(async move {
        subscriber
            .run(move |value| {
                let rcvd = received_clone2.clone();
                async move {
                    if value.get("event_type").and_then(|v| v.as_str()) == Some("test_event") {
                        rcvd.store(true, Ordering::SeqCst);
                    }
                }
            })
            .await
    });

    // Give the subscriber time to create consumer group and start reading
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Add a test message to the stream
    let test_payload = json!({
        "event_type": "test_event",
        "tool": { "name": "integration-test-tool" },
    });
    let _: String = publisher
        .xadd(
            "test_stream_integration",
            false,
            None::<()>,
            "*",
            vec![("payload", test_payload.to_string())],
        )
        .await
        .unwrap();

    // Wait for the message to be received
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if received_clone.load(Ordering::SeqCst) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Cancel the subscriber by dropping the client
    client.quit().await.ok();
    let _ = tokio::time::timeout(Duration::from_secs(2), sub_handle).await;

    // Clean up
    let _: String = publisher
        .xgroup_delconsumer("test_stream_integration", "test_group", "test_consumer")
        .await
        .unwrap_or(String::new());
    let _: String = publisher
        .xgroup_destroy("test_stream_integration", "test_group")
        .await
        .unwrap_or(String::new());
    let _ = publisher.del::<(), _>(&["test_stream_integration"]).await;

    assert!(
        received.load(Ordering::SeqCst),
        "Handler was not called — the subscriber did not receive the stream message"
    );
}
