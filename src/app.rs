use crate::api_client::ApiClient;
use crate::config::Config;
use crate::redis_subscriber::RedisSubscriber;
use tokio::signal;
use tracing::{error, info};

/// Validate critical production settings, returning warnings.
#[must_use]
pub fn validate_config(config: &Config) -> Vec<String> {
    let mut warnings = Vec::new();
    if std::env::var("JWT_SECRET_KEY").is_err() {
        warnings.push(
            "JWT_SECRET_KEY is not set. Using test default — DO NOT use in production!".to_string(),
        );
    }
    if config.gateway_url.starts_with("http://localhost")
        || config.gateway_url.starts_with("http://127.0.0.1")
    {
        warnings
            .push("Gateway URL points to localhost. This may not work in production.".to_string());
    }
    warnings
}

/// Log startup information.
pub fn log_startup(config: &Config) {
    info!("Starting Redis Bridge v{}", env!("CARGO_PKG_VERSION"));
    info!("Redis URL: {}", config.redis_url);
    info!("Stream Key: {}", config.redis_stream);
    info!("Consumer Group: {}", config.redis_stream_group);
    info!("Consumer Name: {}", config.redis_stream_consumer);
    info!("Gateway URL: {}", config.gateway_url);
    info!("Tool Endpoint: {}", config.tool_endpoint);
}

/// Create the application components from configuration.
///
/// # Errors
///
/// Returns an error if the API client fails to initialize.
pub fn create_app(config: &Config) -> anyhow::Result<(ApiClient, RedisSubscriber)> {
    let api_client = ApiClient::new(config.clone())?;
    let subscriber = RedisSubscriber::new(config.clone());
    Ok((api_client, subscriber))
}

/// Build a shutdown signal future that resolves on Ctrl+C or SIGTERM.
///
/// # Panics
///
/// Panics if the Ctrl+C signal handler cannot be installed.
/// On Unix, also panics if the SIGTERM handler cannot be installed.
pub async fn shutdown_signal() {
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
        () = ctrl_c => info!("Received Ctrl+C, shutting down gracefully..."),
        () = terminate => info!("Received SIGTERM, shutting down gracefully..."),
    }
}

/// Calculate exponential backoff with a 60-second cap.
#[must_use]
pub fn calculate_backoff(attempt: u32) -> u64 {
    u64::from(std::cmp::min(5 * 2u32.pow(attempt.min(4)), 60))
}

/// Handle a single notification by creating a tool via the API.
pub async fn handle_message(api_client: &ApiClient, json_value: serde_json::Value) {
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
