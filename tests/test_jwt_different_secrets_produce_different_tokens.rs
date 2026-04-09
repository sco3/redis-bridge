use redis_bridge::jwt::{self, JwtConfig};

#[test]
fn test_jwt_different_secrets_produce_different_tokens() {
    let config1 = JwtConfig {
        secret: "secret-one-that-is-long-enough-for-hmac".to_string(),
        ..JwtConfig::default()
    };
    let config2 = JwtConfig {
        secret: "secret-two-that-is-long-enough-for-hmac".to_string(),
        ..JwtConfig::default()
    };

    let token1 = jwt::generate_jwt_token(&config1).unwrap();
    let token2 = jwt::generate_jwt_token(&config2).unwrap();

    let parts1: Vec<&str> = token1.split('.').collect();
    let parts2: Vec<&str> = token2.split('.').collect();
    assert_ne!(parts1[2], parts2[2]);
}
