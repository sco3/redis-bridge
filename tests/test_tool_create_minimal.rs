use redis_bridge::schemas::ToolCreate;

#[test]
fn test_tool_create_minimal() {
    let json = r#"{"name": "minimal-tool"}"#;
    let tool: ToolCreate = serde_json::from_str(json).unwrap();
    assert_eq!(tool.name, "minimal-tool");
    assert_eq!(tool.integration_type, "REST");
    assert_eq!(tool.request_type, "SSE");
}
