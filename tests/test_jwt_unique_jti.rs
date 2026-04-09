use redis_bridge::jwt::{self, JwtConfig};

#[test]
fn test_jwt_unique_jti() {
    let config = JwtConfig::default();
    let token1 = jwt::generate_jwt_token(&config).unwrap();
    let token2 = jwt::generate_jwt_token(&config).unwrap();

    let parts1: Vec<&str> = token1.split('.').collect();
    let parts2: Vec<&str> = token2.split('.').collect();
    assert_ne!(parts1[2], parts2[2]);
}
