use redis_bridge::schemas::{AuthenticationValues, BasicAuth, BearerAuth};

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
