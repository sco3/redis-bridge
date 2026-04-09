use clap::Parser;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;

#[test]
fn test_api_client_custom_config() {
    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url",
        "http://custom-gateway:8080",
        "--jwt-secret",
        "custom-secret",
        "--jwt-username",
        "user@test.com",
    ])
    .unwrap();
    let client = ApiClient::new(config);
    assert!(client.is_ok());
}
