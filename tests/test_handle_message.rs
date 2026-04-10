use clap::Parser;
use mockito::Server;
use redis_bridge::app;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;

#[tokio::test]
async fn test_handle_message_success() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/tools")
        .with_status(200)
        .with_body(r#"{"id": "123", "name": "test-tool"}"#)
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

    let json_value = serde_json::json!({
        "name": "test-tool",
        "url": "http://example.com/api"
    });

    app::handle_message(&api_client, json_value).await;

    mock.assert_async().await;
}

#[tokio::test]
async fn test_handle_message_error() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/tools")
        .with_status(500)
        .with_body("Internal Server Error")
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

    let json_value = serde_json::json!({
        "name": "test-tool",
        "url": "http://example.com/api"
    });

    // Should not panic even on error
    app::handle_message(&api_client, json_value).await;

    mock.assert_async().await;
}
