use clap::Parser;
use redis_bridge::config::Config;

#[test]
fn test_custom_config_via_cli() {
    let config = Config::try_parse_from([
        "redis-bridge",
        "--redis-url", "redis://custom:6380",
        "--gateway-url", "http://gateway:9000",
        "--jwt-secret", "super-secret",
        "--jwt-username", "user@example.com",
        "--tool-visibility", "private",
        "--tool-integration-type", "MCP",
    ])
    .unwrap();

    assert_eq!(config.redis_url, "redis://custom:6380");
    assert_eq!(config.gateway_url, "http://gateway:9000");
    assert_eq!(config.jwt_secret, "super-secret");
    assert_eq!(config.jwt_username, "user@example.com");
    assert_eq!(config.tool_visibility, "private");
    assert_eq!(config.tool_integration_type, "MCP");
}
