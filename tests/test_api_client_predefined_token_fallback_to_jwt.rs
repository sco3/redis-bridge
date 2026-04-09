use clap::Parser;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;

#[test]
fn test_api_client_predefined_token_fallback_to_jwt() {
    let config = Config::try_parse_from([
        "redis-bridge",
        "--use-predefined-token",
    ])
    .unwrap();
    let client = ApiClient::new(config).unwrap();
    let token = client.get_auth_token();
    assert!(token.is_ok());
    let token_str = token.unwrap();
    let parts: Vec<&str> = token_str.split('.').collect();
    assert_eq!(parts.len(), 3);
}
