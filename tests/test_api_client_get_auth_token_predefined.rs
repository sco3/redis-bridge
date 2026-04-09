use clap::Parser;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;

#[test]
fn test_api_client_get_auth_token_predefined() {
    let config = Config::try_parse_from([
        "redis-bridge",
        "--use-predefined-token",
        "--bearer-token", "my-predefined-token",
    ])
    .unwrap();
    let client = ApiClient::new(config).unwrap();
    let token = client.get_auth_token();
    assert!(token.is_ok());
    assert_eq!(token.unwrap(), "my-predefined-token");
}
