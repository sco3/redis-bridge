use redis_bridge::schemas::ToolCreate;

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
