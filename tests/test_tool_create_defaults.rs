use redis_bridge::schemas::ToolCreate;

#[test]
fn test_tool_create_defaults() {
    let tool = ToolCreate::default();
    assert_eq!(tool.name, String::new());
    assert_eq!(tool.integration_type, "REST");
    assert_eq!(tool.request_type, "SSE");
    assert_eq!(
        tool.input_schema,
        serde_json::json!({"type": "object", "properties": {}})
    );
    assert!(tool.output_schema.is_none());
    assert!(tool.url.is_none());
    assert!(tool.headers.is_none());
    assert!(tool.auth.is_none());
    assert!(tool.gateway_id.is_none());
    assert!(tool.tags.is_empty());
    assert!(tool.visibility.is_none());
}
