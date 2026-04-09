use redis_bridge::redis_subscriber::RedisError;

#[test]
fn test_redis_error_display() {
    let err = RedisError::Subscription("timeout".to_string());
    assert!(err.to_string().contains("Failed to subscribe"));
    assert!(err.to_string().contains("timeout"));

    let err = RedisError::ParseError(serde_json::from_str::<()>("bad").unwrap_err());
    assert!(err.to_string().contains("Failed to parse message"));
}
