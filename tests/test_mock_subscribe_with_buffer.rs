use fred::mocks::Buffer;
use fred::prelude::*;
use std::sync::Arc;

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

    client.subscribe("test_channel").await.unwrap();

    let commands = buffer.take();
    assert!(!commands.is_empty());
    assert_eq!(commands[0].cmd, "SUBSCRIBE");
}
