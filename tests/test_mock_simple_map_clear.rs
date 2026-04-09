use fred::mocks::SimpleMap;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_simple_map_clear() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = RedisConfig {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = RedisClient::new(config, None, None, None);
    client.connect();
    client.wait_for_connect().await.unwrap();

    let _: () = client.set("key1", "val1", None, None, false).await.unwrap();
    let _: () = client.set("key2", "val2", None, None, false).await.unwrap();

    simple_map.clear();

    let val1: Option<String> = client.get("key1").await.unwrap();
    let val2: Option<String> = client.get("key2").await.unwrap();
    assert!(val1.is_none());
    assert!(val2.is_none());
}
