use fred::mocks::Echo;
use fred::prelude::*;
use std::sync::Arc;

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
