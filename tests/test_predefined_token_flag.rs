use clap::Parser;
use redis_bridge::config::Config;

#[test]
fn test_predefined_token_flag() {
    let config = Config::try_parse_from([
        "redis-bridge",
        "--use-predefined-token",
        "--bearer-token", "my-token",
    ])
    .unwrap();
    assert!(config.use_predefined_token);
    assert_eq!(config.bearer_token, Some("my-token".to_string()));
}
