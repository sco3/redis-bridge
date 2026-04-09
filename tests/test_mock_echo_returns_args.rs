use fred::mocks::Echo;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_echo_returns_args() {
    let config = RedisConfig {
        mocks: Some(Arc::new(Echo)),
        ..Default::default()
    };
    let client = RedisClient::new(config, None, None, None);
    client.connect();
    client.wait_for_connect().await.unwrap();

    let result: Vec<RedisValue> = client.set("foo", "bar", None, None, false).await.unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0], RedisValue::String("foo".into()));
    assert_eq!(result[1], RedisValue::String("bar".into()));
}
