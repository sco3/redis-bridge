use redis_bridge::schemas::ToolCreate;
use std::collections::HashMap;

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
        output_schema: Some(serde_json::json!({"type": "object"})),
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
