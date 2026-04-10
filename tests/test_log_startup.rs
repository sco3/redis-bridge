use clap::Parser;
use redis_bridge::app;
use redis_bridge::config::Config;
use std::sync::{Arc, Mutex};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct TestWriter {
    logs: Arc<Mutex<Vec<String>>>,
}

impl std::io::Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            self.logs.lock().unwrap().push(s.to_string());
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_log_startup() {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();
    let layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_target(false)
        .with_level(false)
        .with_writer(move || TestWriter {
            logs: logs_clone.clone(),
        })
        .compact();

    let _guard = tracing_subscriber::registry().with(layer).set_default();

    let config = Config::try_parse_from(["redis-bridge"]).unwrap();
    app::log_startup(&config);

    let captured = logs.lock().unwrap();
    let all_logs = captured.join("\n");
    assert!(all_logs.contains("Starting Redis Bridge"));
    assert!(all_logs.contains(&config.redis_url));
    assert!(all_logs.contains(&config.redis_stream));
    assert!(all_logs.contains(&config.redis_stream_group));
    assert!(all_logs.contains(&config.redis_stream_consumer));
    assert!(all_logs.contains(&config.gateway_url));
    assert!(all_logs.contains(&config.tool_endpoint));
}
