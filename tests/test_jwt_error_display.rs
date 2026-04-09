use redis_bridge::jwt;

#[test]
fn test_jwt_error_display() {
    let err = jwt::JwtError::HmacInitialization;
    assert!(err.to_string().contains("HMAC initialization failed"));
}
