use clap::Parser;
use redis_bridge::config::Config;

#[test]
fn test_gateway_base_url_trims_slashes() {
    let config =
        Config::try_parse_from(["redis-bridge", "--gateway-url", "http://localhost:8080/"])
            .unwrap();
    assert_eq!(config.gateway_base_url(), "http://localhost:8080");

    let config =
        Config::try_parse_from(["redis-bridge", "--gateway-url", "http://localhost:8080///"])
            .unwrap();
    assert_eq!(config.gateway_base_url(), "http://localhost:8080");
}
