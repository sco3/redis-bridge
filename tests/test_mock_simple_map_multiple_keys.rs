use fred::mocks::SimpleMap;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_simple_map_multiple_keys() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let _: () = client
        .set("user:1", "Alice", None, None, false)
        .await
        .unwrap();
    let _: () = client
        .set("user:2", "Bob", None, None, false)
        .await
        .unwrap();

    let user1: String = client.get("user:1").await.unwrap();
    let user2: String = client.get("user:2").await.unwrap();
    assert_eq!(user1, "Alice");
    assert_eq!(user2, "Bob");

    let count: i64 = client.del("user:1").await.unwrap();
    assert_eq!(count, 1);

    let user1_after: Option<String> = client.get("user:1").await.unwrap();
    assert!(user1_after.is_none());

    let user2_still: String = client.get("user:2").await.unwrap();
    assert_eq!(user2_still, "Bob");
}
