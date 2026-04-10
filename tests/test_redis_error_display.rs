use redis_bridge::redis_subscriber::RedisError;

#[test]
fn test_redis_error_display() {
    let err =
        RedisError::ConsumerGroupCreation("BUSYGROUP Consumer Group already exists".to_string());
    assert!(err.to_string().contains("Failed to create consumer group"));
    assert!(err.to_string().contains("BUSYGROUP"));

    let err = RedisError::ParseError(serde_json::from_str::<()>("bad").unwrap_err());
    assert!(err.to_string().contains("Failed to parse message"));
}
