use fred::mocks::SimpleMap;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_simple_map_set_get() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let _: () = client.set("foo", "bar", None, None, false).await.unwrap();
    let val: String = client.get("foo").await.unwrap();
    assert_eq!(val, "bar");
}
