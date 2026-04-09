use clap::Parser;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;

#[test]
fn test_api_client_creation() {
    let config = Config::try_parse_from(["redis-bridge"]).unwrap();
    let client = ApiClient::new(config);
    assert!(client.is_ok());
}
