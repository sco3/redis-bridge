use clap::Parser;
use redis_bridge::config::Config;

#[test]
fn test_tool_creation_url() {
    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url", "http://localhost:4444",
        "--tool-endpoint", "/tools",
    ])
    .unwrap();
    assert_eq!(config.tool_creation_url(), "http://localhost:4444/tools");

    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url", "http://localhost:4444/",
        "--tool-endpoint", "/api/v1/tools",
    ])
    .unwrap();
    assert_eq!(config.tool_creation_url(), "http://localhost:4444/api/v1/tools");
}
