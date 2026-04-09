use redis_bridge::api_client::ApiError;

#[test]
fn test_api_error_display() {
    let err = ApiError::ApiError {
        status: 404,
        message: "Not found".to_string(),
    };
    assert!(err.to_string().contains("404"));
    assert!(err.to_string().contains("Not found"));

    let err = ApiError::Serialization(serde_json::from_str::<()>("invalid").unwrap_err());
    assert!(err.to_string().contains("Tool serialization failed"));
}
