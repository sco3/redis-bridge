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
use reqwest::Client as HttpClient;
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

    let tool_suffix = args.tool_suffix.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
    });

    let tool_name = format!("smoke-test-tool-{tool_suffix}");

    print_banner();

    let redis_client = connect_redis(&args.config).await;
    let bridge = spawn_bridge(&args.config).await;
    publish_test_event(&redis_client, &args.config, &tool_name).await;
    let found = verify_tool_created(&args.config, &tool_name, args.verify_timeout_secs, args.poll_interval_secs).await;
    cleanup_bridge(bridge).await;
    print_summary(found);
}

fn print_banner() {
    println!();
    log("═══════════════════════════════════════════");
    log(" redis-bridge  Smoke Test");
    log("═══════════════════════════════════════════");
    println!();
}

async fn connect_redis(cfg: &Config) -> fred::clients::Client {
    log("Checking Redis...");
    let redis_cfg = fred::types::config::Config::from_url(&cfg.redis_url).unwrap_or_else(|e| {
        fail(&format!("Invalid Redis URL '{}': {}", cfg.redis_url, e));
        std::process::exit(1);
    });
    let client = Builder::from_config(redis_cfg).build().unwrap_or_else(|e| {
        fail(&format!("Failed to build Redis client: {e}"));
        std::process::exit(1);
    });
    client.init().await.unwrap_or_else(|e| {
        fail(&format!("Failed to connect to Redis: {e}"));
        std::process::exit(1);
    });
    ok(&format!("Redis ready at {}", cfg.redis_url));
    client
}

async fn spawn_bridge(cfg: &Config) -> tokio::process::Child {
    log("Launching redis-bridge...");
    let bridge = Command::new(
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
        fail(&format!("Failed to spawn redis-bridge: {e}"));
        std::process::exit(1);
    });
    log(&format!("Bridge PID: {}", bridge.id().unwrap_or(0)));
    tokio::time::sleep(Duration::from_secs(2)).await;
    ok("Bridge is running");
    bridge
}

async fn publish_test_event(client: &fred::clients::Client, cfg: &Config, tool_name: &str) {
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
    let _result: Value = client
        .publish(&cfg.redis_channel, payload_str.clone())
        .await
        .unwrap_or_else(|e| {
            fail(&format!("Failed to publish to Redis: {e}"));
            std::process::exit(1);
        });
    ok("Published event (payload sent)");
    tokio::time::sleep(Duration::from_secs(2)).await;
}

async fn verify_tool_created(cfg: &Config, tool_name: &str, timeout_secs: u64, poll_interval_secs: u64) -> bool {
    log(&format!(
        "Polling {}/tools for tool '{}'...",
        cfg.gateway_url, tool_name
    ));

    let http_client = HttpClient::builder()
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

    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let poll_interval = Duration::from_secs(poll_interval_secs);

    while Instant::now() < deadline {
        let resp = http_client
            .get(format!("{}/tools", cfg.gateway_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await;

        match resp {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.text().await.unwrap_or_default();
                if body.contains(tool_name) {
                    return true;
                }
            }
            _ => {}
        }

        tokio::time::sleep(poll_interval).await;
    }

    false
}

async fn cleanup_bridge(mut bridge: tokio::process::Child) {
    bridge.start_kill().ok();
    let _ = bridge.wait().await;
}

fn print_summary(found: bool) {
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
