use fred::mocks::Buffer;
use fred::prelude::*;
use std::sync::Arc;

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
