use redis_bridge::schemas::ToolCreate;
use std::collections::HashMap;

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
