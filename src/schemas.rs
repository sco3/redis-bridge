use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthenticationValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic: Option<BasicAuth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<BearerAuth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BearerAuth {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_integration_type")]
    pub integration_type: String,
    #[serde(default = "default_request_type")]
    pub request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(
        default = "default_input_schema",
        rename = "inputSchema",
        alias = "input_schema"
    )]
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none", alias = "output_schema", rename = "outputSchema")]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub jsonpath_filter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthenticationValues>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_mapping: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_mapping: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(default = "default_expose_passthrough")]
    pub expose_passthrough: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowlist: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_chain_pre: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_chain_post: Option<Vec<String>>,
}

impl Default for ToolCreate {
    fn default() -> Self {
        Self {
            name: String::new(),
            display_name: None,
            title: None,
            url: None,
            description: None,
            integration_type: default_integration_type(),
            request_type: default_request_type(),
            headers: None,
            input_schema: default_input_schema(),
            output_schema: None,
            annotations: HashMap::new(),
            jsonpath_filter: String::new(),
            auth: None,
            gateway_id: None,
            tags: Vec::new(),
            team_id: None,
            owner_email: None,
            visibility: None,
            base_url: None,
            path_template: None,
            query_mapping: None,
            header_mapping: None,
            timeout_ms: None,
            expose_passthrough: default_expose_passthrough(),
            allowlist: None,
            plugin_chain_pre: None,
            plugin_chain_post: None,
        }
    }
}

fn default_integration_type() -> String {
    "REST".to_string()
}

fn default_request_type() -> String {
    "SSE".to_string()
}

fn default_input_schema() -> serde_json::Value {
    serde_json::json!({"type": "object", "properties": {}})
}

fn default_expose_passthrough() -> bool {
    true
}

impl ToolCreate {
    /// Create a [`ToolCreate`] from a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON value cannot be deserialized.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Convert this [`ToolCreate`] to a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the struct cannot be serialized.
    pub fn to_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

