use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use redis_bridge::jwt::{self, JwtClaims, JwtConfig, JwtHeader};

#[test]
fn test_jwt_generation() {
    let config = JwtConfig::default();
    let token = jwt::generate_jwt_token(&config).unwrap();

    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);

    let header_bytes = URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
    let header: JwtHeader = serde_json::from_slice(&header_bytes).unwrap();
    assert_eq!(header.typ, "JWT");

    let claims_bytes = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
    let claims: JwtClaims = serde_json::from_slice(&claims_bytes).unwrap();
    assert_eq!(claims.sub, config.username);
    assert_eq!(claims.token_use, "session");
    assert!(claims.user.is_admin);
}
