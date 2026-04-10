use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "redis-bridge")]
#[command(version = "0.1.0")]
#[command(about = "Redis to REST API bridge for MCP Gateway tool creation", long_about = None)]
pub struct Config {
    /// Redis URL to connect to
    #[arg(
        short,
        long,
        env = "REDIS_URL",
        default_value = "redis://127.0.0.1:6379"
    )]
    pub redis_url: String,

    /// Redis channel to subscribe to for notifications
    #[arg(
        short = 'c',
        long,
        env = "REDIS_CHANNEL",
        default_value = "tool_notifications"
    )]
    pub redis_channel: String,

    /// Base URL of the MCP Gateway API
    #[arg(
        short,
        long,
        env = "GATEWAY_URL",
        default_value = "http://localhost:8080"
    )]
    pub gateway_url: String,

    /// JWT secret key for token generation
    #[arg(
        long,
        env = "JWT_SECRET_KEY",
        default_value = "my-test-key-but-now-longer-than-32-bytes"
    )]
    pub jwt_secret: String,

    /// JWT username/subject for token generation
    #[arg(long, env = "JWT_USERNAME", default_value = "admin@example.com")]
    pub jwt_username: String,

    /// JWT audience claim
    #[arg(long, env = "JWT_AUDIENCE", default_value = "mcpgateway-api")]
    pub jwt_audience: String,

    /// JWT issuer claim
    #[arg(long, env = "JWT_ISSUER", default_value = "mcpgateway")]
    pub jwt_issuer: String,

    /// JWT signing algorithm
    #[arg(long, env = "JWT_ALGORITHM", default_value = "HS256")]
    pub jwt_algorithm: String,

    /// Tool creation endpoint path
    #[arg(long, env = "TOOL_ENDPOINT", default_value = "/tools")]
    pub tool_endpoint: String,

    /// Visibility level for created tools (private, team, public)
    #[arg(long, env = "TOOL_VISIBILITY", default_value = "public")]
    pub tool_visibility: String,

    /// Integration type for tools (REST, MCP, A2A)
    #[arg(long, env = "TOOL_INTEGRATION_TYPE", default_value = "REST")]
    pub tool_integration_type: String,

    /// Request type for REST tools (GET, POST, PUT, DELETE, PATCH)
    #[arg(long, env = "TOOL_REQUEST_TYPE", default_value = "POST")]
    pub tool_request_type: String,

    /// Whether to generate JWT tokens or use a pre-shared token
    #[arg(long, env = "USE_PREDEFINED_TOKEN")]
    pub use_predefined_token: bool,

    /// Pre-defined bearer token (if not using JWT generation)
    #[arg(long, env = "BEARER_TOKEN")]
    pub bearer_token: Option<String>,
}

impl Config {
    #[must_use]
    pub fn gateway_base_url(&self) -> String {
        self.gateway_url.trim_end_matches('/').to_string()
    }

    #[must_use]
    pub fn tool_creation_url(&self) -> String {
        format!("{}{}", self.gateway_base_url(), self.tool_endpoint)
    }
}
