use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("HMAC initialization failed")]
    HmacInitialization,
    #[error("Serialization failed")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtHeader {
    alg: String,
    typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    exp: i64,
    iat: i64,
    aud: String,
    iss: String,
    jti: String,
    token_use: String,
    user: JwtUser,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtUser {
    email: String,
    full_name: String,
    is_admin: bool,
    auth_provider: String,
}

pub struct JwtConfig {
    pub secret: String,
    pub username: String,
    pub audience: String,
    pub issuer: String,
    pub algorithm: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET_KEY")
                .unwrap_or_else(|_| "my-test-key-but-now-longer-than-32-bytes".to_string()),
            username: std::env::var("JWT_USERNAME")
                .unwrap_or_else(|_| "admin@example.com".to_string()),
            audience: std::env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "mcpgateway-api".to_string()),
            issuer: std::env::var("JWT_ISSUER")
                .unwrap_or_else(|_| "mcpgateway".to_string()),
            algorithm: std::env::var("JWT_ALGORITHM").unwrap_or_else(|_| "HS256".to_string()),
        }
    }
}

fn base64_encode_json<T: Serialize>(value: &T) -> Result<String, JwtError> {
    let json = serde_json::to_string(value)?;
    Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
}

/// Generate a JWT token using the provided configuration.
///
/// # Errors
///
/// Returns an error if HMAC initialization fails or serialization fails.
pub fn generate_jwt_token(config: &JwtConfig) -> Result<String, JwtError> {
    let now = Utc::now();
    let exp = now + chrono::Duration::hours(8760); // 1 year

    let header = JwtHeader {
        alg: config.algorithm.clone(),
        typ: "JWT".to_string(),
    };

    let jti: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let claims = JwtClaims {
        sub: config.username.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        aud: config.audience.clone(),
        iss: config.issuer.clone(),
        jti,
        token_use: "session".to_string(),
        user: JwtUser {
            email: config.username.clone(),
            full_name: "Rust MCP Bridge".to_string(),
            is_admin: true,
            auth_provider: "local".to_string(),
        },
    };

    let header_b64 = base64_encode_json(&header)?;
    let claims_b64 = base64_encode_json(&claims)?;

    let message = format!("{header_b64}.{claims_b64}");

    let mut mac = HmacSha256::new_from_slice(config.secret.as_bytes())
        .map_err(|_| JwtError::HmacInitialization)?;
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let signature_b64 = URL_SAFE_NO_PAD.encode(result.into_bytes());

    Ok(format!("{message}.{signature_b64}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation() {
        let config = JwtConfig::default();
        let token = generate_jwt_token(&config).unwrap();

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        // Decode and verify the header
        let header_bytes = URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
        let header: JwtHeader = serde_json::from_slice(&header_bytes).unwrap();
        assert_eq!(header.typ, "JWT");

        // Decode and verify the claims
        let claims_bytes = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
        let claims: JwtClaims = serde_json::from_slice(&claims_bytes).unwrap();
        assert_eq!(claims.sub, config.username);
        assert_eq!(claims.token_use, "session");
        assert!(claims.user.is_admin);
    }
}
