use fred::mocks::SimpleMap;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_simple_map_del() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = RedisConfig {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = RedisClient::new(config, None, None, None);
    client.connect();
    client.wait_for_connect().await.unwrap();

    let _: () = client
        .set("key1", "value1", None, None, false)
        .await
        .unwrap();
    let count: i64 = client.del("key1").await.unwrap();
    assert_eq!(count, 1);

    let val: Option<String> = client.get("key1").await.unwrap();
    assert!(val.is_none());
}
