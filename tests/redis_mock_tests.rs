// Integration tests for Redis mocking functionality using fred v10
use fred::mocks::{Buffer, Echo, MockCommand, Mocks, SimpleMap};
use fred::prelude::*;
use std::sync::Arc;

/// Test basic string operations with `SimpleMap` mock
#[tokio::test]
async fn test_mock_string_operations() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    // Test SET
    let _: () = client
        .set("test_key", "test_value", None, None, false)
        .await
        .unwrap();

    // Test GET
    let value: String = client.get("test_key").await.unwrap();
    assert_eq!(value, "test_value");
}

/// Test multiple sequential operations
#[tokio::test]
async fn test_mock_sequential_operations() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let _: () = client.set("counter", "0", None, None, false).await.unwrap();
    let _: () = client.set("counter", "1", None, None, false).await.unwrap();
    let _: () = client.set("counter", "2", None, None, false).await.unwrap();

    let value: String = client.get("counter").await.unwrap();
    assert_eq!(value, "2");
}

/// Test hash operations - Echo returns the arguments
#[tokio::test]
async fn test_mock_hash_operations() {
    let config = Config {
        mocks: Some(Arc::new(Echo)),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    // hset takes a RedisMap (HashMap of field->value pairs)
    // Echo returns the args, so we get back a Vec of Value
    let mut map = std::collections::HashMap::new();
    map.insert("name", "John Doe");
    let result: Vec<Value> = client.hset("user:1000", map).await.unwrap();

    // Echo returns [key, field1, value1, ...]
    assert!(!result.is_empty());
}

/// Test stream operations with Buffer
#[tokio::test]
async fn test_mock_stream_with_buffer() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let _: String = client
        .xadd(
            "notifications",
            false,
            None::<()>,
            "*",
            vec![("payload", "Hello stream!")],
        )
        .await
        .unwrap();

    let commands = buffer.take();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].cmd, "XADD");
}

/// Test JSON message handling in pub/sub context
#[tokio::test]
async fn test_mock_json_message() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let message = serde_json::json!({
        "event": "tool_created",
        "tool_name": "weather_api",
        "timestamp": "2026-04-09T10:00:00Z"
    });

    let _: () = client
        .set("notification:1", message.to_string(), None, None, false)
        .await
        .unwrap();

    let stored: String = client.get("notification:1").await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stored).unwrap();
    assert_eq!(parsed["event"], "tool_created");
    assert_eq!(parsed["tool_name"], "weather_api");
}

/// Test error handling with custom mock
#[derive(Debug)]
struct ErrorMock;

impl Mocks for ErrorMock {
    fn process_command(&self, _command: MockCommand) -> Result<Value, Error> {
        Err(Error::new(ErrorKind::NotFound, "ERR no such key"))
    }
}

#[tokio::test]
async fn test_mock_error_handling() {
    let config = Config {
        mocks: Some(Arc::new(ErrorMock)),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    let result: Result<String, _> = client.get("missing_key").await;
    assert!(result.is_err());
}

/// Test list operations with Buffer
#[tokio::test]
async fn test_mock_list_operations() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    let _: String = client.lpush("tasks", "task1").await.unwrap();
    let _: String = client.lpush("tasks", "task2").await.unwrap();
    let _: String = client.llen("tasks").await.unwrap();

    let commands = buffer.take();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].cmd, "LPUSH");
    assert_eq!(commands[1].cmd, "LPUSH");
    assert_eq!(commands[2].cmd, "LLEN");
}

/// Test set operations with Buffer
#[tokio::test]
async fn test_mock_set_operations() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    let _: String = client.sadd("tags", "rust").await.unwrap();
    let _: String = client.sadd("tags", "redis").await.unwrap();
    let _: String = client.scard("tags").await.unwrap();

    let commands = buffer.take();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].cmd, "SADD");
    assert_eq!(commands[1].cmd, "SADD");
    assert_eq!(commands[2].cmd, "SCARD");
}

/// Test key expiration with Buffer
#[tokio::test]
async fn test_mock_expiration() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    let _: () = client
        .set("session:abc", "active", None, None, false)
        .await
        .unwrap();
    let _: () = client.expire("session:abc", 3600, None).await.unwrap();
    let _: Value = client.ttl("session:abc").await.unwrap();

    let commands = buffer.take();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].cmd, "SET");
    assert_eq!(commands[1].cmd, "EXPIRE");
    assert_eq!(commands[2].cmd, "TTL");
}

/// Test multiple clients with independent mocks
#[tokio::test]
async fn test_mock_multiple_clients() {
    let map1 = Arc::new(SimpleMap::new());
    let config1 = Config {
        mocks: Some(map1.clone()),
        ..Default::default()
    };

    let client1 = Builder::from_config(config1).build().unwrap();
    client1.init().await.unwrap();

    // Set and get with first client
    let _: () = client1
        .set("client1_key", "value1", None, None, false)
        .await
        .unwrap();
    let val1: String = client1.get("client1_key").await.unwrap();
    assert_eq!(val1, "value1");

    // Create second client with its own map
    let map2 = Arc::new(SimpleMap::new());
    let config2 = Config {
        mocks: Some(map2.clone()),
        ..Default::default()
    };

    let client2 = Builder::from_config(config2).build().unwrap();
    client2.init().await.unwrap();

    let _: () = client2
        .set("client2_key", "value2", None, None, false)
        .await
        .unwrap();
    let val2: String = client2.get("client2_key").await.unwrap();
    assert_eq!(val2, "value2");

    // Verify isolation: client1's map doesn't have client2's key
    let client1_val: Option<String> = client1.get("client2_key").await.unwrap();
    assert!(client1_val.is_none());
}

/// Test complex JSON payload in tool notification scenario
#[tokio::test]
async fn test_mock_tool_notification_scenario() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    // Simulate a realistic tool notification payload
    let tool_notification = serde_json::json!({
        "event_type": "tool_created",
        "tool": {
            "name": "get_weather",
            "url": "http://api.weather.com/v1",
            "description": "Fetch weather data for a location",
            "integrationType": "REST",
            "requestType": "GET",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name or zip code"
                    }
                },
                "required": ["location"]
            }
        },
        "metadata": {
            "created_at": "2026-04-09T10:30:00Z",
            "created_by": "admin@example.com"
        }
    });

    let _: () = client
        .set(
            "tool:notification:1",
            tool_notification.to_string(),
            None,
            None,
            false,
        )
        .await
        .unwrap();

    // Verify we can parse the message back
    let stored: String = client.get("tool:notification:1").await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stored).unwrap();
    assert_eq!(parsed["event_type"], "tool_created");
    assert_eq!(parsed["tool"]["name"], "get_weather");
}

/// Test Echo mock returns arguments
#[tokio::test]
async fn test_echo_mock_returns_args() {
    let config = Config {
        mocks: Some(Arc::new(Echo)),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    let result: Vec<Value> = client.set("foo", "bar", None, None, false).await.unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], Value::String("foo".into()));
    assert_eq!(result[1], Value::String("bar".into()));
}

/// Test Buffer mock clear and len
#[tokio::test]
async fn test_buffer_clear() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    let _: String = client.set("foo", "bar", None, None, false).await.unwrap();
    assert_eq!(buffer.len(), 1);

    buffer.clear();
    assert_eq!(buffer.len(), 0);
}

/// Test pipeline with Echo mock
#[tokio::test]
async fn test_pipeline_with_echo() {
    let config = Config {
        mocks: Some(Arc::new(Echo)),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    let pipeline = client.pipeline();
    pipeline.get::<(), _>("foo").await.unwrap();
    pipeline.get::<(), _>("bar").await.unwrap();

    let all: Vec<Vec<String>> = pipeline.all().await.unwrap();
    assert_eq!(all, vec![vec!["foo"], vec!["bar"]]);
}

/// Test stream commands are recorded
#[tokio::test]
async fn test_stream_xgroup_recorded() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    // xgroup_create gets split into XGROUP CREATE command
    let _: String = client
        .xgroup_create("test_stream", "test_group", "0", true)
        .await
        .unwrap();

    let commands = buffer.take();
    assert!(!commands.is_empty());
    assert_eq!(commands[0].cmd, "XGROUP");
}

/// Test `SimpleMap` stores and retrieves multiple values
#[tokio::test]
async fn test_simple_map_multiple_values() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();

    client.init().await.unwrap();

    // Set multiple keys
    for i in 0..10 {
        let _: () = client
            .set(
                format!("key:{i}"),
                format!("value:{i}"),
                None,
                None,
                false,
            )
            .await
            .unwrap();
    }

    // Retrieve and verify all
    for i in 0..10 {
        let val: String = client.get(format!("key:{i}")).await.unwrap();
        assert_eq!(val, format!("value:{i}"));
    }
}
