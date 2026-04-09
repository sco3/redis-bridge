use redis_bridge::api_client::ApiError;
use redis_bridge::jwt::JwtError;

#[test]
fn test_api_error_jwt_display() {
    let err = ApiError::Jwt(JwtError::HmacInitialization);
    assert!(err.to_string().contains("JWT generation failed"));
}
