use redis_bridge::schemas::ToolCreate;

#[test]
fn test_tool_create_from_value_invalid() {
    let bad_value = serde_json::json!({"name": 123});
    assert!(ToolCreate::from_value(bad_value).is_err());
}
