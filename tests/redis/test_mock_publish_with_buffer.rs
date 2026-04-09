use fred::mocks::Buffer;
use fred::prelude::*;
use std::sync::Arc;

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

    let _: String = client.publish("notifications", "hello").await.unwrap();

    let commands = buffer.take();
    assert!(!commands.is_empty());
    assert_eq!(commands[0].cmd, "PUBLISH");
}
