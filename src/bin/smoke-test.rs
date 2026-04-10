//! smoke-test — End-to-end smoke test for redis-bridge.
//!
//! Spawns the bridge as a subprocess, publishes a test event to Redis,
//! and polls the gateway REST API to verify the tool was created.
//!
//! Usage:
//! ```bash
//! cargo run --bin smoke-test
//!
//! # Custom gateway and Redis
//! REDIS_URL=redis://myredis:6379 GATEWAY_URL=http://myhost:4444 cargo run --bin smoke-test
//! ```

use std::time::{Duration, Instant};

use clap::Parser;
use fred::prelude::*;
use redis_bridge::config::Config;
use redis_bridge::jwt::{self, JwtConfig};
use reqwest::Client;
use serde_json::json;
use tokio::process::Command;

#[derive(Parser, Debug)]
#[command(name = "smoke-test")]
#[command(about = "End-to-end smoke test for redis-bridge")]
struct Args {
    #[command(flatten)]
    config: Config,

    /// Maximum seconds to wait for the tool to appear
    #[arg(long, default_value = "15")]
    verify_timeout_secs: u64,

    /// Poll interval in seconds
    #[arg(long, default_value = "1")]
    poll_interval_secs: u64,

    /// Unique tool name suffix (default: epoch timestamp)
    #[arg(long)]
    tool_suffix: Option<String>,
}

struct Colors;

impl Colors {
    fn cyan() -> &'static str {
        "\x1b[0;36m"
    }
    fn green() -> &'static str {
        "\x1b[0;32m"
    }
    fn red() -> &'static str {
        "\x1b[0;31m"
    }
    fn reset() -> &'static str {
        "\x1b[0m"
    }
}

fn log(msg: &str) {
    println!("{}[smoke]{}  {}", Colors::cyan(), Colors::reset(), msg);
}
fn ok(msg: &str) {
    println!("{}[PASS]{}   {}", Colors::green(), Colors::reset(), msg);
}
fn fail(msg: &str) {
    println!("{}[FAIL]{}   {}", Colors::red(), Colors::reset(), msg);
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let cfg = &args.config;

    let tool_suffix = args.tool_suffix.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
    });

    let tool_name = format!("smoke-test-tool-{}", tool_suffix);

    println!();
    log("═══════════════════════════════════════════");
    log(" redis-bridge  Smoke Test");
    log("═══════════════════════════════════════════");
    println!();

    // ─── Step 0: Verify Redis connectivity ─────────────────────────────
    log("Checking Redis...");
    let redis_cfg = fred::types::config::Config::from_url(&cfg.redis_url).unwrap_or_else(|e| {
        fail(&format!("Invalid Redis URL '{}': {}", cfg.redis_url, e));
        std::process::exit(1);
    });
    let redis_client = Builder::from_config(redis_cfg).build().unwrap_or_else(|e| {
        fail(&format!("Failed to build Redis client: {}", e));
        std::process::exit(1);
    });
    redis_client.init().await.unwrap_or_else(|e| {
        fail(&format!("Failed to connect to Redis: {}", e));
        std::process::exit(1);
    });
    ok(&format!("Redis ready at {}", cfg.redis_url));

    // ─── Step 1: Spawn redis-bridge ────────────────────────────────────
    log("Launching redis-bridge...");
    let mut bridge = Command::new(
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("redis-bridge"),
    )
    .env("REDIS_URL", &cfg.redis_url)
    .env("REDIS_CHANNEL", &cfg.redis_channel)
    .env("GATEWAY_URL", &cfg.gateway_url)
    .env("JWT_SECRET_KEY", &cfg.jwt_secret)
    .env("JWT_USERNAME", &cfg.jwt_username)
    .env("JWT_AUDIENCE", &cfg.jwt_audience)
    .env("JWT_ISSUER", &cfg.jwt_issuer)
    .env("JWT_ALGORITHM", &cfg.jwt_algorithm)
    .env("RUST_LOG", "redis_bridge=warn")
    .kill_on_drop(true)
    .spawn()
    .unwrap_or_else(|e| {
        fail(&format!("Failed to spawn redis-bridge: {}", e));
        std::process::exit(1);
    });
    log(&format!("Bridge PID: {}", bridge.id().unwrap_or(0)));

    // Give the bridge time to connect and subscribe
    tokio::time::sleep(Duration::from_secs(2)).await;
    ok("Bridge is running");

    // ─── Step 2: Publish test event to Redis ───────────────────────────
    let payload = json!({
        "tool": {
            "name": tool_name,
            "url": "http://smoke-test.internal/api",
            "description": "Created by rust smoke test",
            "integrationType": "REST",
            "requestType": "GET"
        }
    });
    let payload_str = serde_json::to_string(&payload).unwrap();

    log(&format!(
        "Publishing test event to Redis channel '{}'...",
        cfg.redis_channel
    ));
    let _result: Value = redis_client
        .publish(&cfg.redis_channel, payload_str.clone())
        .await
        .unwrap_or_else(|e| {
            fail(&format!("Failed to publish to Redis: {}", e));
            std::process::exit(1);
        });
    ok(&"Published event (payload sent)".to_string());

    // Give the bridge time to process
    tokio::time::sleep(Duration::from_secs(2)).await;

    // ─── Step 3: Verify via REST API ───────────────────────────────────
    log(&format!(
        "Polling {}/tools for tool '{}'...",
        cfg.gateway_url, tool_name
    ));

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    let jwt_cfg = JwtConfig {
        secret: cfg.jwt_secret.clone(),
        username: cfg.jwt_username.clone(),
        audience: cfg.jwt_audience.clone(),
        issuer: cfg.jwt_issuer.clone(),
        algorithm: cfg.jwt_algorithm.clone(),
        ..Default::default()
    };
    let token = jwt::generate_jwt_token(&jwt_cfg).expect("Failed to generate JWT");

    let deadline = Instant::now() + Duration::from_secs(args.verify_timeout_secs);
    let poll_interval = Duration::from_secs(args.poll_interval_secs);

    let mut found = false;
    while Instant::now() < deadline {
        let resp = client
            .get(format!("{}/tools", cfg.gateway_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;

        match resp {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.text().await.unwrap_or_default();
                if body.contains(&tool_name) {
                    found = true;
                    break;
                }
            }
            _ => {}
        }

        tokio::time::sleep(poll_interval).await;
    }

    // ─── Cleanup: kill bridge ──────────────────────────────────────────
    bridge.start_kill().ok();
    let _ = bridge.wait().await;

    // ─── Summary ────────────────────────────────────────────────────────
    println!();
    log("═══════════════════════════════════════════");
    if found {
        ok("Smoke test passed");
    } else {
        fail("Smoke test failed");
    }
    log("═══════════════════════════════════════════");
    println!();

    if !found {
        std::process::exit(1);
    }
}
