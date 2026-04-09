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

#[allow(non_snake_case)]
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

    #[test]
    fn test_tool_create_full_roundtrip() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());

        let mut annotations = HashMap::new();
        annotations.insert("title".to_string(), serde_json::json!("My Tool"));

        let original = ToolCreate {
            name: "full-tool".to_string(),
            display_name: Some("Full Tool".to_string()),
            title: Some("Full Tool Title".to_string()),
            url: Some("http://example.com/api".to_string()),
            description: Some("A fully featured tool".to_string()),
            integration_type: "REST".to_string(),
            request_type: "GET".to_string(),
            headers: Some(headers),
            input_schema: serde_json::json!({"type": "object", "properties": {"query": {"type": "string"}}}),
            outputSchema: Some(serde_json::json!({"type": "object"})),
            annotations,
            jsonpath_filter: "$.data".to_string(),
            auth: None,
            gateway_id: Some("gw-123".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            team_id: Some("team-1".to_string()),
            owner_email: Some("owner@example.com".to_string()),
            visibility: Some("team".to_string()),
            base_url: Some("http://example.com".to_string()),
            path_template: Some("/api/{id}".to_string()),
            query_mapping: Some(serde_json::json!({"q": "query"})),
            header_mapping: Some(serde_json::json!({"Auth": "token"})),
            timeout_ms: Some(5000),
            expose_passthrough: true,
            allowlist: Some(vec!["example.com".to_string()]),
            plugin_chain_pre: Some(vec!["auth".to_string()]),
            plugin_chain_post: Some(vec!["log".to_string()]),
        };

        let value = original.to_value().unwrap();
        let restored = ToolCreate::from_value(value).unwrap();

        assert_eq!(restored.name, original.name);
        assert_eq!(restored.display_name, original.display_name);
        assert_eq!(restored.url, original.url);
        assert_eq!(restored.description, original.description);
        assert_eq!(restored.integration_type, original.integration_type);
        assert_eq!(restored.request_type, original.request_type);
        assert_eq!(restored.gateway_id, original.gateway_id);
        assert_eq!(restored.tags, original.tags);
        assert_eq!(restored.visibility, original.visibility);
        assert_eq!(restored.timeout_ms, original.timeout_ms);
    }

    #[test]
    fn test_tool_create_defaults() {
        let tool = ToolCreate::default();
        assert_eq!(tool.name, String::new());
        assert_eq!(tool.integration_type, "REST");
        assert_eq!(tool.request_type, "SSE");
        assert_eq!(tool.input_schema, serde_json::json!({"type": "object", "properties": {}}));
        assert!(tool.outputSchema.is_none());
        assert!(tool.url.is_none());
        assert!(tool.headers.is_none());
        assert!(tool.auth.is_none());
        assert!(tool.gateway_id.is_none());
        assert!(tool.tags.is_empty());
        assert!(tool.visibility.is_none());
    }

    #[test]
    fn test_tool_create_camelcase_serialization() {
        let tool = ToolCreate {
            name: "snake-test".to_string(),
            integration_type: "MCP".to_string(),
            request_type: "POST".to_string(),
            jsonpath_filter: "$.result".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&tool).unwrap();
        // Verify camelCase in output
        assert!(json.contains("integrationType"));
        assert!(json.contains("requestType"));
        assert!(json.contains("jsonpathFilter"));
        assert!(json.contains("inputSchema"));
    }

    #[test]
    fn test_tool_create_from_value_invalid() {
        let bad_value = serde_json::json!({"name": 123}); // name should be string
        assert!(ToolCreate::from_value(bad_value).is_err());
    }

    #[test]
    fn test_tool_create_minimal() {
        let json = r#"{"name": "minimal-tool"}"#;
        let tool: ToolCreate = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "minimal-tool");
        assert_eq!(tool.integration_type, "REST");
        assert_eq!(tool.request_type, "SSE");
    }

    #[test]
    fn test_authentication_values() {
        let auth = AuthenticationValues {
            basic: Some(BasicAuth {
                username: "user".to_string(),
                password: "pass".to_string(),
            }),
            bearer: None,
            custom: None,
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("basic"));

        let auth_bearer = AuthenticationValues {
            basic: None,
            bearer: Some(BearerAuth {
                token: "tok123".to_string(),
            }),
            custom: None,
        };

        let json = serde_json::to_string(&auth_bearer).unwrap();
        assert!(json.contains("bearer"));
    }

    #[test]
    fn test_tool_create_with_annotations() {
        let mut annotations = HashMap::new();
        annotations.insert("readOnlyHint".to_string(), serde_json::json!(true));
        annotations.insert("destructiveHint".to_string(), serde_json::json!(false));

        let tool = ToolCreate {
            name: "annotated-tool".to_string(),
            annotations,
            ..Default::default()
        };

        let value = tool.to_value().unwrap();
        assert!(value.get("annotations").is_some());
    }
}
