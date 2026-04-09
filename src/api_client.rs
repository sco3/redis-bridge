use reqwest::Client;
use thiserror::Error;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::jwt::{self, JwtConfig};
use crate::schemas::ToolCreate;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JWT generation failed: {0}")]
    Jwt(#[from] jwt::JwtError),
    #[error("Tool serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    config: Config,
}

impl ApiClient {
    /// Create a new API client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client fails to initialize.
    pub fn new(config: Config) -> Result<Self, ApiError> {
        let client = Client::builder()
            .build()
            .map_err(ApiError::Http)?;

        Ok(Self { client, config })
    }

    fn get_auth_token(&self) -> Result<String, ApiError> {
        if self.config.use_predefined_token {
            if let Some(token) = &self.config.bearer_token {
                info!("Using pre-defined bearer token");
                return Ok(token.clone());
            }
            warn!("No bearer token provided, falling back to JWT generation");
        }

        let jwt_config = JwtConfig {
            secret: self.config.jwt_secret.clone(),
            username: self.config.jwt_username.clone(),
            audience: self.config.jwt_audience.clone(),
            issuer: self.config.jwt_issuer.clone(),
            algorithm: self.config.jwt_algorithm.clone(),
        };

        let token = jwt::generate_jwt_token(&jwt_config)?;
        info!("Generated JWT token successfully");
        Ok(token)
    }

    /// Create a tool via the MCP Gateway API.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the API returns an error status.
    pub async fn create_tool(&self, tool: &ToolCreate) -> Result<serde_json::Value, ApiError> {
        let token = self.get_auth_token()?;
        let url = self.config.tool_creation_url();

        info!("Sending tool creation request to: {}", url);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .json(tool)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let body = response.json::<serde_json::Value>().await?;
            info!("Tool created successfully");
            Ok(body)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "No error body".to_string());

            error!(
                "Tool creation failed with status {}: {}",
                status, error_text
            );

            Err(ApiError::ApiError {
                status: status.as_u16(),
                message: error_text,
            })
        }
    }

    /// Create a tool from a raw JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON cannot be deserialized into a `ToolCreate`
    /// or if the tool creation request fails.
    pub async fn create_tool_from_json(
        &self,
        json_value: serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let tool = ToolCreate::from_value(json_value.clone()).map_err(|e| {
            error!("Failed to parse ToolCreate from JSON: {}", e);
            ApiError::Serialization(e)
        })?;

        self.create_tool(&tool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_api_error_display() {
        let err = ApiError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        assert!(err.to_string().contains("404"));
        assert!(err.to_string().contains("Not found"));

        let err = ApiError::Serialization(serde_json::from_str::<()>("invalid").unwrap_err());
        assert!(err.to_string().contains("Tool serialization failed"));
    }

    #[test]
    fn test_api_error_jwt_display() {
        let err = ApiError::Jwt(jwt::JwtError::HmacInitialization);
        assert!(err.to_string().contains("JWT generation failed"));
    }

    #[test]
    fn test_api_client_creation() {
        let config = Config::try_parse_from(["redis-bridge"]).unwrap();
        let client = ApiClient::new(config);
        assert!(client.is_ok());
    }
}
