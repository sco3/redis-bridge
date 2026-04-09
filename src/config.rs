use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "redis-bridge")]
#[command(version = "0.1.0")]
#[command(about = "Redis to REST API bridge for MCP Gateway tool creation", long_about = None)]
pub struct Config {
    /// Redis URL to connect to
    #[arg(short, long, env = "REDIS_URL", default_value = "redis://127.0.0.1:6379")]
    pub redis_url: String,

    /// Redis channel to subscribe to for notifications
    #[arg(short = 'c', long, env = "REDIS_CHANNEL", default_value = "tool_notifications")]
    pub redis_channel: String,

    /// Base URL of the MCP Gateway API
    #[arg(
        short,
        long,
        env = "GATEWAY_URL",
        default_value = "http://localhost:4444"
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
    #[arg(
        long,
        env = "JWT_USERNAME",
        default_value = "admin@example.com"
    )]
    pub jwt_username: String,

    /// JWT audience claim
    #[arg(
        long,
        env = "JWT_AUDIENCE",
        default_value = "mcpgateway-api"
    )]
    pub jwt_audience: String,

    /// JWT issuer claim
    #[arg(
        long,
        env = "JWT_ISSUER",
        default_value = "mcpgateway"
    )]
    pub jwt_issuer: String,

    /// JWT signing algorithm
    #[arg(
        long,
        env = "JWT_ALGORITHM",
        default_value = "HS256"
    )]
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
        format!(
            "{}{}",
            self.gateway_base_url(),
            self.tool_endpoint
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_default_config() {
        let config = Config::try_parse_from(["redis-bridge"]).unwrap();
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(config.redis_channel, "tool_notifications");
        assert_eq!(config.gateway_url, "http://localhost:4444");
        assert_eq!(config.tool_endpoint, "/tools");
        assert_eq!(config.jwt_secret, "my-test-key-but-now-longer-than-32-bytes");
        assert_eq!(config.jwt_username, "admin@example.com");
        assert_eq!(config.jwt_audience, "mcpgateway-api");
        assert_eq!(config.jwt_issuer, "mcpgateway");
        assert_eq!(config.jwt_algorithm, "HS256");
        assert_eq!(config.tool_visibility, "public");
        assert_eq!(config.tool_integration_type, "REST");
        assert_eq!(config.tool_request_type, "POST");
        assert!(!config.use_predefined_token);
        assert!(config.bearer_token.is_none());
    }

    #[test]
    fn test_custom_config_via_cli() {
        let config = Config::try_parse_from([
            "redis-bridge",
            "--redis-url", "redis://custom:6380",
            "--gateway-url", "http://gateway:9000",
            "--jwt-secret", "super-secret",
            "--jwt-username", "user@example.com",
            "--tool-visibility", "private",
            "--tool-integration-type", "MCP",
        ])
        .unwrap();

        assert_eq!(config.redis_url, "redis://custom:6380");
        assert_eq!(config.gateway_url, "http://gateway:9000");
        assert_eq!(config.jwt_secret, "super-secret");
        assert_eq!(config.jwt_username, "user@example.com");
        assert_eq!(config.tool_visibility, "private");
        assert_eq!(config.tool_integration_type, "MCP");
    }

    #[test]
    fn test_gateway_base_url_trims_slashes() {
        let config = Config::try_parse_from([
            "redis-bridge",
            "--gateway-url", "http://localhost:4444/",
        ])
        .unwrap();
        assert_eq!(config.gateway_base_url(), "http://localhost:4444");

        let config = Config::try_parse_from([
            "redis-bridge",
            "--gateway-url", "http://localhost:4444///",
        ])
        .unwrap();
        assert_eq!(config.gateway_base_url(), "http://localhost:4444");
    }

    #[test]
    fn test_tool_creation_url() {
        let config = Config::try_parse_from([
            "redis-bridge",
            "--gateway-url", "http://localhost:4444",
            "--tool-endpoint", "/tools",
        ])
        .unwrap();
        assert_eq!(config.tool_creation_url(), "http://localhost:4444/tools");

        let config = Config::try_parse_from([
            "redis-bridge",
            "--gateway-url", "http://localhost:4444/",
            "--tool-endpoint", "/api/v1/tools",
        ])
        .unwrap();
        assert_eq!(config.tool_creation_url(), "http://localhost:4444/api/v1/tools");
    }

    #[test]
    fn test_predefined_token_flag() {
        let config = Config::try_parse_from([
            "redis-bridge",
            "--use-predefined-token",
            "--bearer-token", "my-token",
        ])
        .unwrap();
        assert!(config.use_predefined_token);
        assert_eq!(config.bearer_token, Some("my-token".to_string()));
    }
}
