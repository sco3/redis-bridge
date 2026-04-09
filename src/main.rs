use clap::Parser;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;
use redis_bridge::redis_subscriber::RedisSubscriber;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("redis_bridge=info")
        }))
        .init();

    // Parse CLI config (supports both CLI args and env vars)
    let config = Config::parse();

    // Validate critical production settings
    if std::env::var("JWT_SECRET_KEY").is_err() {
        warn!("JWT_SECRET_KEY is not set. Using test default — DO NOT use in production!");
    }

    info!("Starting Redis Bridge v{}", env!("CARGO_PKG_VERSION"));
    info!("Redis URL: {}", config.redis_url);
    info!("Redis Channel: {}", config.redis_channel);
    info!("Gateway URL: {}", config.gateway_url);
    info!("Tool Endpoint: {}", config.tool_endpoint);

    // Create API client
    let api_client = ApiClient::new(config.clone())?;

    // Create Redis subscriber
    let subscriber = RedisSubscriber::new(config.clone());

    // Run the subscription loop with reconnection and graceful shutdown
    info!("Starting Redis subscription loop...");

    let mut attempt: u32 = 0;

    loop {
        let api_client = api_client.clone();
        let shutdown_fut = async {
            let ctrl_c = async {
                signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to install SIGTERM handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            tokio::select! {
                _ = ctrl_c => info!("Received Ctrl+C, shutting down gracefully..."),
                _ = terminate => info!("Received SIGTERM, shutting down gracefully..."),
            }
        };

        tokio::select! {
            result = subscriber.run(move |json_value| {
                let api_client = api_client.clone();
                async move {
                    info!("Processing notification...");
                    match api_client.create_tool_from_json(json_value.clone()).await {
                        Ok(response) => {
                            info!("Tool created successfully: {:?}", response);
                        }
                        Err(e) => {
                            error!("Failed to create tool: {}", e);
                        }
                    }
                }
            }) => {
                if let Err(e) = result {
                    attempt += 1;
                    let backoff = std::cmp::min(5 * 2u32.pow(attempt.min(4)), 60);
                    error!("Redis subscription failed: {} (attempt {})", e, attempt);
                    warn!("Reconnecting in {} seconds...", backoff);
                    tokio::time::sleep(std::time::Duration::from_secs(backoff as u64)).await;
                } else {
                    info!("Redis subscription ended normally.");
                    break;
                }
            }
            _ = shutdown_fut => {
                info!("Shutdown requested, exiting.");
                break;
            }
        }
    }

    Ok(())
}
