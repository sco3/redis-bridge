use clap::Parser;
use mockito::Server;
use redis_bridge::api_client::{ApiClient, ApiError};
use redis_bridge::config::Config;

/// Test `create_tool` when the API returns an error with an empty response body.
/// Covers the error handling path where the response body is empty.
#[tokio::test]
async fn test_create_tool_error_empty_body() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/tools")
        .with_status(500)
        .with_body("")
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

    let tool = redis_bridge::schemas::ToolCreate {
        name: "test-tool".to_string(),
        url: Some("http://example.com/api".to_string()),
        ..Default::default()
    };

    let result = api_client.create_tool(&tool).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ApiError::ApiError { status, message } => {
            assert_eq!(status, 500);
            assert_eq!(message, "");
        }
        e => panic!("Expected ApiError, got: {e:?}"),
    }

    mock.assert_async().await;
}
