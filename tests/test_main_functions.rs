use clap::Parser;
use redis_bridge::app;
use redis_bridge::config::Config;
use serial_test::serial;

fn set_env(key: &str, val: &str) {
    // SAFETY: tests are serialized with #[serial], no concurrent env access
    unsafe { std::env::set_var(key, val) };
}

fn unset_env(key: &str) {
    // SAFETY: tests are serialized with #[serial]
    unsafe { std::env::remove_var(key) };
}

#[test]
#[serial]
fn test_validate_config_no_warnings_with_jwt() {
    unset_env("JWT_SECRET_KEY");
    set_env("JWT_SECRET_KEY", "production-secret-that-is-long-enough");
    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url", "https://api.example.com",
    ])
    .unwrap();

    let warnings = app::validate_config(&config);
    assert!(warnings.is_empty(), "Expected no warnings, got: {:?}", warnings);

    unset_env("JWT_SECRET_KEY");
}

#[test]
#[serial]
fn test_validate_config_warns_on_missing_jwt_secret() {
    unset_env("JWT_SECRET_KEY");
    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url", "https://api.example.com",
    ])
    .unwrap();

    let warnings = app::validate_config(&config);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("JWT_SECRET_KEY is not set"));
}

#[test]
#[serial]
fn test_validate_config_warns_on_localhost_gateway() {
    unset_env("JWT_SECRET_KEY");
    set_env("JWT_SECRET_KEY", "some-secret-key-that-is-long-enough");
    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url", "http://localhost:8080",
    ])
    .unwrap();

    let warnings = app::validate_config(&config);
    assert_eq!(warnings.len(), 1, "Expected 1 warning, got: {:?}", warnings);
    assert!(warnings[0].contains("localhost"));

    unset_env("JWT_SECRET_KEY");
}

#[test]
#[serial]
fn test_validate_config_multiple_warnings() {
    unset_env("JWT_SECRET_KEY");
    let config = Config::try_parse_from([
        "redis-bridge",
        "--gateway-url", "http://127.0.0.1:4444",
    ])
    .unwrap();

    let warnings = app::validate_config(&config);
    assert_eq!(warnings.len(), 2, "Expected 2 warnings, got: {:?}", warnings);
    assert!(warnings[0].contains("JWT_SECRET_KEY"));
    assert!(warnings[1].contains("localhost") || warnings[1].contains("127.0.0.1"));
}

#[test]
fn test_calculate_backoff_first_attempt() {
    assert_eq!(app::calculate_backoff(0), 5);
}

#[test]
fn test_calculate_backoff_exponential() {
    assert_eq!(app::calculate_backoff(1), 10);
    assert_eq!(app::calculate_backoff(2), 20);
    assert_eq!(app::calculate_backoff(3), 40);
}

#[test]
fn test_calculate_backoff_capped_at_60() {
    assert_eq!(app::calculate_backoff(4), 60);
    assert_eq!(app::calculate_backoff(5), 60);
    assert_eq!(app::calculate_backoff(100), 60);
}

#[test]
fn test_create_app_success() {
    let config = Config::try_parse_from(["redis-bridge"]).unwrap();
    let result = app::create_app(&config);
    assert!(result.is_ok());
    let (api_client, subscriber) = result.unwrap();
    assert_eq!(subscriber.redis_url(), config.redis_url);
    assert_eq!(subscriber.redis_channel(), config.redis_channel);
    drop(api_client);
}
