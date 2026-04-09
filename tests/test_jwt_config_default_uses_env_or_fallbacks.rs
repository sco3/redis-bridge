use redis_bridge::jwt::JwtConfig;

#[test]
fn test_jwt_config_default_uses_env_or_fallbacks() {
    let config = JwtConfig::default();
    assert_eq!(config.secret, "my-test-key-but-now-longer-than-32-bytes");
    assert_eq!(config.username, "admin@example.com");
    assert_eq!(config.audience, "mcpgateway-api");
    assert_eq!(config.issuer, "mcpgateway");
    assert_eq!(config.algorithm, "HS256");
    assert_eq!(config.token_ttl_hours, 1);
    assert!(config.is_admin);
}
