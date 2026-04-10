use clap::Parser;
use redis_bridge::app;
use redis_bridge::config::Config;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the tracing subscriber with sensible defaults.
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("redis_bridge=info")),
        )
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    init_tracing();

    // Parse CLI config (supports both CLI args and env vars)
    let config = Config::parse();

    // Validate critical production settings
    for warning in app::validate_config(&config) {
        warn!("{}", warning);
    }

    app::log_startup(&config);

    // Create application components
    let (api_client, subscriber) = app::create_app(&config)?;

    // Run the subscription loop with reconnection and graceful shutdown
    info!("Starting Redis subscription loop...");

    let mut attempt: u32 = 0;

    loop {
        let api_client = api_client.clone();
        tokio::select! {
            result = subscriber.run(move |json_value| {
                let api_client = api_client.clone();
                async move {
                    app::handle_message(&api_client, json_value).await;
                }
            }) => {
                if let Err(e) = result {
                    attempt += 1;
                    let backoff = app::calculate_backoff(attempt);
                    error!("Redis subscription failed: {} (attempt {})", e, attempt);
                    warn!("Reconnecting in {} seconds...", backoff);
                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                } else {
                    info!("Redis subscription ended normally.");
                    break;
                }
            }
            () = app::shutdown_signal() => {
                info!("Shutdown requested, exiting.");
                break;
            }
        }
    }

    Ok(())
}
