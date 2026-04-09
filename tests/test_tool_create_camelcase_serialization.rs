use redis_bridge::schemas::ToolCreate;

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
    assert!(json.contains("integrationType"));
    assert!(json.contains("requestType"));
    assert!(json.contains("jsonpathFilter"));
    assert!(json.contains("inputSchema"));
}
