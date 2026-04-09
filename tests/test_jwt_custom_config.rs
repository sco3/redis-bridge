use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use redis_bridge::jwt::{self, JwtClaims, JwtConfig, JwtHeader};

#[test]
fn test_jwt_custom_config() {
    let config = JwtConfig {
        secret: "my-custom-secret-key-that-is-long-enough".to_string(),
        username: "custom-user@test.com".to_string(),
        audience: "custom-audience".to_string(),
        issuer: "custom-issuer".to_string(),
        algorithm: "HS256".to_string(),
        token_ttl_hours: 24,
        is_admin: false,
    };

    let token = jwt::generate_jwt_token(&config).unwrap();
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);

    let header_bytes = URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
    let header: JwtHeader = serde_json::from_slice(&header_bytes).unwrap();
    assert_eq!(header.alg, "HS256");

    let claims_bytes = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
    let claims: JwtClaims = serde_json::from_slice(&claims_bytes).unwrap();
    assert_eq!(claims.sub, "custom-user@test.com");
    assert_eq!(claims.aud, "custom-audience");
    assert_eq!(claims.iss, "custom-issuer");
    assert_eq!(claims.user.email, "custom-user@test.com");
    assert_eq!(claims.user.full_name, "Rust MCP Bridge");
    assert!(!claims.user.is_admin);
}
