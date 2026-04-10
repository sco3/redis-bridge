use clap::Parser;
use mockito::Server;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;

#[tokio::test]
async fn test_create_tool_from_json_wrapped_format() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/tools")
        .with_status(200)
        .with_body(r#"{"id": "456", "name": "wrapped-tool"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--tool-endpoint",
        "/tools",
    ])
    .unwrap();

    let api_client = ApiClient::new(config).unwrap();

    // Wrapped format: {"tool": {...}} — the client should unwrap this
    let wrapped_json = serde_json::json!({
        "tool": {
            "name": "wrapped-tool",
            "url": "http://example.com/api"
        }
    });

    let result = api_client.create_tool_from_json(wrapped_json).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["id"], "456");

    mock.assert_async().await;
}
