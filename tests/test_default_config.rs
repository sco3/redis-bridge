use clap::Parser;
use redis_bridge::config::Config;

#[test]
fn test_default_config() {
    let config = Config::try_parse_from(["redis-bridge"]).unwrap();
    assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
    assert_eq!(config.redis_channel, "tool_notifications");
    assert_eq!(config.gateway_url, "http://localhost:4444");
    assert_eq!(config.tool_endpoint, "/tools");
    assert_eq!(
        config.jwt_secret,
        "my-test-key-but-now-longer-than-32-bytes"
    );
    assert_eq!(config.jwt_username, "admin@example.com");
    assert_eq!(config.jwt_audience, "mcpgateway-api");
    assert_eq!(config.jwt_issuer, "mcpgateway");
    assert_eq!(config.jwt_algorithm, "HS256");
    assert_eq!(config.tool_visibility, "public");
    assert_eq!(config.tool_integration_type, "REST");
    assert_eq!(config.tool_request_type, "POST");
    assert!(!config.use_predefined_token);
    assert!(config.bearer_token.is_none());
}
