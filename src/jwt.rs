use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use rand::RngExt;
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
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub aud: String,
    pub iss: String,
    pub jti: String,
    pub token_use: String,
    pub user: JwtUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtUser {
    pub email: String,
    pub full_name: String,
    pub is_admin: bool,
    pub auth_provider: String,
}

pub struct JwtConfig {
    pub secret: String,
    pub username: String,
    pub audience: String,
    pub issuer: String,
    pub algorithm: String,
    /// Token TTL in hours. Defaults to 1 hour.
    pub token_ttl_hours: i64,
    /// Whether the token grants admin privileges.
    pub is_admin: bool,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: jwt_secret_from_env(),
            username: std::env::var("JWT_USERNAME")
                .unwrap_or_else(|_| "admin@example.com".to_string()),
            audience: std::env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "mcpgateway-api".to_string()),
            issuer: std::env::var("JWT_ISSUER").unwrap_or_else(|_| "mcpgateway".to_string()),
            algorithm: std::env::var("JWT_ALGORITHM").unwrap_or_else(|_| "HS256".to_string()),
            token_ttl_hours: 1,
            is_admin: true,
        }
    }
}

/// Read JWT secret from env, falling back to a known test key.
/// In production, the caller MUST validate that `JWT_SECRET_KEY` is set
/// (see `main.rs` validation).
fn jwt_secret_from_env() -> String {
    std::env::var("JWT_SECRET_KEY")
        .unwrap_or_else(|_| "my-test-key-but-now-longer-than-32-bytes".to_string())
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
    let exp = now + chrono::Duration::hours(config.token_ttl_hours);

    let header = JwtHeader {
        alg: config.algorithm.clone(),
        typ: "JWT".to_string(),
    };

    let jti: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
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
            is_admin: config.is_admin,
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
