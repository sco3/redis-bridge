use fred::mocks::Echo;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_pipeline_with_echo() {
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
