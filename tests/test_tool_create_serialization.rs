use redis_bridge::schemas::ToolCreate;

#[test]
fn test_tool_create_serialization() {
    let tool = ToolCreate {
        name: "test-tool".to_string(),
        url: Some("http://localhost:8080/tool".to_string()),
        description: Some("A test tool".to_string()),
        integration_type: "REST".to_string(),
        request_type: "POST".to_string(),
        output_schema: None,
        ..Default::default()
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("test-tool"));
    assert!(json.contains("http://localhost:8080/tool"));
}
