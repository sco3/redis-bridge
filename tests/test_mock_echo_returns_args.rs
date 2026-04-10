use fred::mocks::Echo;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_echo_returns_args() {
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
