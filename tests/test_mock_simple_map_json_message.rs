use fred::mocks::SimpleMap;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_simple_map_json_message() {
    let simple_map = Arc::new(SimpleMap::new());
    let config = Config {
        mocks: Some(simple_map.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let json_value = serde_json::json!({
        "type": "tool_created",
        "name": "weather_api",
        "version": "1.0.0"
    });

    let _: () = client
        .set("notification:1", json_value.to_string(), None, None, false)
        .await
        .unwrap();

    let stored: String = client.get("notification:1").await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stored).unwrap();
    assert_eq!(parsed["type"], "tool_created");
    assert_eq!(parsed["name"], "weather_api");
}
