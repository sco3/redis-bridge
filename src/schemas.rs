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
    #[allow(non_snake_case)]
    pub outputSchema: Option<serde_json::Value>,
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
            outputSchema: None,
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
    /// Create a ToolCreate from a JSON value
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Convert to JSON value for sending
    pub fn to_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_create_serialization() {
        let tool = ToolCreate {
            name: "test-tool".to_string(),
            url: Some("http://localhost:8080/tool".to_string()),
            description: Some("A test tool".to_string()),
            integration_type: "REST".to_string(),
            request_type: "POST".to_string(),
            outputSchema: None,
            ..Default::default()
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("test-tool"));
        assert!(json.contains("http://localhost:8080/tool"));
    }

    #[test]
    fn test_tool_create_deserialization() {
        let json = r#"{
            "name": "test-tool",
            "url": "http://localhost:8080/tool",
            "description": "A test tool",
            "integrationType": "REST",
            "requestType": "POST"
        }"#;

        let tool: ToolCreate = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "test-tool");
        assert_eq!(tool.url, Some("http://localhost:8080/tool".to_string()));
    }
}
