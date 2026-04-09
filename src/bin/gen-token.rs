//! gen-token — Minimal standalone JWT token generator for MCP Gateway.
//!
//! A thin wrapper around `redis_bridge::jwt` that requires no project config.
//! Useful for quick token generation in scripts, CI, or debugging.
//!
//! Usage:
//! ```bash
//! # Generate token with defaults
//! cargo run --bin gen-token
//!
//! # Custom secret and username
//! JWT_SECRET_KEY=my-secret JWT_USERNAME=ops@example.com cargo run --bin gen-token
//!
//! # All options via CLI or env
//! cargo run --bin gen-token -- \
//!   --jwt-secret my-secret \
//!   --jwt-username ops@test.com \
//!   --jwt-audience my-api \
//!   --jwt-issuer my-service \
//!   --jwt-algorithm HS256 \
//!   --token-ttl-hours 24 \
//!   --is-admin
//!
//! # Raw output only (for piping into other commands)
//! cargo run --bin gen-token -- --raw
//! ```

use clap::Parser;
use redis_bridge::jwt::{self, JwtConfig};

#[derive(Parser, Debug)]
#[command(name = "gen-token")]
#[command(about = "Generate a JWT token for MCP Gateway authentication")]
struct Args {
    /// JWT secret key
    #[arg(long, env = "JWT_SECRET_KEY")]
    jwt_secret: Option<String>,

    /// JWT subject/username
    #[arg(long, env = "JWT_USERNAME")]
    jwt_username: Option<String>,

    /// JWT audience
    #[arg(long, env = "JWT_AUDIENCE")]
    jwt_audience: Option<String>,

    /// JWT issuer
    #[arg(long, env = "JWT_ISSUER")]
    jwt_issuer: Option<String>,

    /// JWT signing algorithm
    #[arg(long, env = "JWT_ALGORITHM")]
    jwt_algorithm: Option<String>,

    /// Token TTL in hours
    #[arg(long, env = "JWT_TOKEN_TTL_HOURS", default_value = "1")]
    token_ttl_hours: i64,

    /// Whether the token grants admin privileges
    #[arg(long, env = "JWT_IS_ADMIN", default_value = "true")]
    is_admin: bool,

    /// Output only the raw token (no metadata)
    #[arg(long, short = 'r')]
    raw: bool,
}

fn main() {
    let args = Args::parse();

    let default = JwtConfig::default();
    let secret = args.jwt_secret.unwrap_or(default.secret);
    let username = args.jwt_username.unwrap_or(default.username);
    let audience = args.jwt_audience.unwrap_or(default.audience);
    let issuer = args.jwt_issuer.unwrap_or(default.issuer);
    let algorithm = args.jwt_algorithm.unwrap_or(default.algorithm);

    let jwt_config = JwtConfig {
        secret,
        username,
        audience,
        issuer,
        algorithm,
        token_ttl_hours: args.token_ttl_hours,
        is_admin: args.is_admin,
    };

    let token = jwt::generate_jwt_token(&jwt_config).unwrap_or_else(|e| {
        eprintln!("Error generating token: {}", e);
        std::process::exit(1);
    });

    if args.raw {
        println!("{}", token);
    } else {
        println!("JWT Token:");
        println!("  {}", token);
        println!();
        println!("Configuration:");
        println!("  Secret:      {}", mask_secret(&jwt_config.secret));
        println!("  Username:    {}", jwt_config.username);
        println!("  Audience:    {}", jwt_config.audience);
        println!("  Issuer:      {}", jwt_config.issuer);
        println!("  Algorithm:   {}", jwt_config.algorithm);
        println!("  TTL:         {} hours", jwt_config.token_ttl_hours);
        println!("  Is Admin:    {}", jwt_config.is_admin);
        println!();
        println!("Use with:");
        println!("  curl -H \"Authorization: Bearer {}\" http://localhost:8080/tools", token);
    }
}

fn mask_secret(secret: &str) -> String {
    if secret.len() <= 8 {
        "****".to_string()
    } else {
        let visible = secret.len().min(6);
        format!("{}{}", &secret[..visible], "*".repeat(secret.len() - visible))
    }
}
