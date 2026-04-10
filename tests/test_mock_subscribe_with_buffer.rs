use fred::mocks::Buffer;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_subscribe_with_buffer() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    client.subscribe("test_channel").await.unwrap();

    let commands = buffer.take();
    assert!(!commands.is_empty());
    assert_eq!(commands[0].cmd, "SUBSCRIBE");
}
