// Integration tests for API client with mockito
use clap::Parser;
use mockito::Server;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;
use redis_bridge::schemas::ToolCreate;
use serde_json::json;
use std::collections::HashMap;

/// Test successful tool creation via API
#[tokio::test]
async fn test_create_tool_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "tool-123", "name": "test-tool", "status": "created"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();
    let tool = ToolCreate {
        name: "test-tool".to_string(),
        url: Some("http://example.com/api".to_string()),
        description: Some("A test tool".to_string()),
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    if let Err(ref e) = result {
        eprintln!("Error: {e:?}");
    }
    assert!(result.is_ok());
    let body = result.unwrap();
    assert_eq!(body["id"], "tool-123");
    assert_eq!(body["name"], "test-tool");

    mock.assert_async().await;
}

/// Test tool creation with authentication values
#[tokio::test]
async fn test_create_tool_with_auth() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "tool-456"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();

    let mut headers = HashMap::new();
    headers.insert("X-API-Key".to_string(), "secret".to_string());

    let tool = ToolCreate {
        name: "auth-tool".to_string(),
        url: Some("http://secure-api.com/endpoint".to_string()),
        description: Some("A tool with auth".to_string()),
        integration_type: "REST".to_string(),
        request_type: "GET".to_string(),
        headers: Some(headers),
        auth: Some(redis_bridge::schemas::AuthenticationValues {
            bearer: Some(redis_bridge::schemas::BearerAuth {
                token: "api-token-123".to_string(),
            }),
            basic: None,
            custom: None,
        }),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["id"], "tool-456");

    mock.assert_async().await;
}

/// Test tool creation failure - server error
#[tokio::test]
async fn test_create_tool_server_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error": "Internal server error"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();
    let tool = ToolCreate {
        name: "failing-tool".to_string(),
        url: Some("http://example.com/api".to_string()),
        description: Some("This will fail".to_string()),
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("500"));
    }

    mock.assert_async().await;
}

/// Test tool creation failure - validation error
#[tokio::test]
async fn test_create_tool_validation_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .with_status(422)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"detail": "Validation error: missing required field"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();
    let tool = ToolCreate {
        name: "invalid-tool".to_string(),
        url: None,
        description: None,
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("422"));
        assert!(e.to_string().contains("Validation error"));
    }

    mock.assert_async().await;
}

/// Test tool creation with custom endpoint
#[tokio::test]
async fn test_create_tool_custom_endpoint() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/tools")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "custom-endpoint-tool"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--tool-endpoint",
        "/api/v1/tools",
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();
    let tool = ToolCreate {
        name: "custom-endpoint-tool".to_string(),
        url: Some("http://example.com/api".to_string()),
        description: Some("Tool with custom endpoint".to_string()),
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["id"], "custom-endpoint-tool");

    mock.assert_async().await;
}

/// Test tool creation with pre-defined bearer token
#[tokio::test]
async fn test_create_tool_with_bearer_token() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .match_header("Authorization", "Bearer my-predefined-token")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "token-auth-tool"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--use-predefined-token",
        "--bearer-token",
        "my-predefined-token",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();
    let tool = ToolCreate {
        name: "token-auth-tool".to_string(),
        url: Some("http://example.com/api".to_string()),
        description: Some("Tool using bearer token auth".to_string()),
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    assert!(result.is_ok());

    mock.assert_async().await;
}

/// Test `create_tool_from_json` with valid JSON
#[tokio::test]
async fn test_create_tool_from_json() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "json-tool-id"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();

    let json_value = json!({
        "name": "json-tool",
        "url": "http://json-api.com/endpoint",
        "description": "Created from JSON",
        "integrationType": "REST",
        "requestType": "GET"
    });

    let result = client.create_tool_from_json(json_value).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["id"], "json-tool-id");

    mock.assert_async().await;
}

/// Test `create_tool_from_json` with invalid JSON
#[tokio::test]
async fn test_create_tool_from_json_invalid() {
    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        "http://localhost:8080",
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();

    // name should be a string, not a number
    let json_value = json!({
        "name": 12345
    });

    let result = client.create_tool_from_json(json_value).await;
    assert!(result.is_err());
}

/// Test tool creation with tags and visibility
#[tokio::test]
async fn test_create_tool_with_tags_and_visibility() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "tagged-tool-id"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();
    let tool = ToolCreate {
        name: "tagged-tool".to_string(),
        url: Some("http://example.com/api".to_string()),
        description: Some("A tool with tags".to_string()),
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        tags: vec![
            "production".to_string(),
            "api".to_string(),
            "v1".to_string(),
        ],
        visibility: Some("public".to_string()),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["id"], "tagged-tool-id");

    mock.assert_async().await;
}

/// Test tool creation with input schema
#[tokio::test]
async fn test_create_tool_with_input_schema() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/tools")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "schema-tool-id"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();
    let tool = ToolCreate {
        name: "schema-tool".to_string(),
        url: Some("http://example.com/api".to_string()),
        description: Some("A tool with input schema".to_string()),
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results",
                    "default": 10
                }
            },
            "required": ["query"]
        }),
        ..Default::default()
    };

    let result = client.create_tool(&tool).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["id"], "schema-tool-id");

    mock.assert_async().await;
}

/// Test multiple sequential tool creations
#[tokio::test]
async fn test_create_multiple_tools() {
    let mut server = Server::new_async().await;

    let mock1 = server
        .mock("POST", "/tools")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "tool-1"}"#)
        .create_async()
        .await;

    let mock2 = server
        .mock("POST", "/tools")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "tool-2"}"#)
        .create_async()
        .await;

    let mock3 = server
        .mock("POST", "/tools")
        .with_status(201)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"id": "tool-3"}"#)
        .create_async()
        .await;

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        &server.url(),
        "--jwt-secret",
        "test-secret-key-that-is-long-enough",
        "--jwt-username",
        "admin@example.com",
    ])
    .unwrap();

    let client = ApiClient::new(config).unwrap();

    for i in 1..=3 {
        let tool = ToolCreate {
            name: format!("tool-{i}"),
            url: Some(format!("http://example.com/api/{i}")),
            description: Some(format!("Tool number {i}")),
            integration_type: "REST".to_string(),
            request_type: "POST".to_string(),
            ..Default::default()
        };

        let result = client.create_tool(&tool).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["id"], format!("tool-{i}"));
    }

    mock1.assert_async().await;
    mock2.assert_async().await;
    mock3.assert_async().await;
}
