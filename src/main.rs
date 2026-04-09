use clap::Parser;
use redis_bridge::api_client::ApiClient;
use redis_bridge::config::Config;
use redis_bridge::redis_subscriber::RedisSubscriber;
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

    info!("Starting Redis Bridge v{}", env!("CARGO_PKG_VERSION"));
    info!("Redis URL: {}", config.redis_url);
    info!("Redis Channel: {}", config.redis_channel);
    info!("Gateway URL: {}", config.gateway_url);
    info!("Tool Endpoint: {}", config.tool_endpoint);

    // Create API client
    let api_client = ApiClient::new(config.clone())?;

    // Create Redis subscriber
    let subscriber = RedisSubscriber::new(config.clone());

    // Run the subscription loop
    info!("Starting Redis subscription loop...");

    let result = subscriber
        .run(move |json_value| {
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
        })
        .await;

    if let Err(e) = result {
        error!("Redis subscription failed: {}", e);
        warn!("Attempting to reconnect in 5 seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        // In production, you might want to implement reconnection logic here
    }

    Ok(())
}
