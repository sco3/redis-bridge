use fred::mocks::{MockCommand, Mocks};
use fred::prelude::*;
use std::sync::Arc;

/// Custom mock implementing a realistic tool-notification scenario
#[derive(Debug)]
struct ToolNotificationMock {
    publish_buffer: std::sync::Mutex<Vec<(String, String)>>,
}

impl ToolNotificationMock {
    fn new() -> Self {
        Self {
            publish_buffer: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn take_published(&self) -> Vec<(String, String)> {
        self.publish_buffer.lock().unwrap().drain(..).collect()
    }
}

impl Mocks for ToolNotificationMock {
    fn process_command(&self, command: MockCommand) -> Result<Value, Error> {
        if &*command.cmd == "PUBLISH" {
            let channel = match command.args.first() {
                Some(Value::String(s)) => s.to_string(),
                Some(Value::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
                _ => {
                    return Err(Error::new(ErrorKind::InvalidArgument, "Invalid channel"));
                }
            };
            let message = match command.args.get(1) {
                Some(Value::String(s)) => s.to_string(),
                Some(Value::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
                _ => {
                    return Err(Error::new(ErrorKind::InvalidArgument, "Invalid message"));
                }
            };
            self.publish_buffer.lock().unwrap().push((channel, message));
            Ok(Value::Integer(1))
        } else if &*command.cmd == "SUBSCRIBE" {
            Ok(Value::Queued)
        } else {
            Err(Error::new(ErrorKind::Unknown, "Unimplemented."))
        }
    }
}

#[tokio::test]
async fn test_mock_custom_tool_notification() {
    let mock = Arc::new(ToolNotificationMock::new());
    let config = Config {
        mocks: Some(mock.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let notification = serde_json::json!({
        "event_type": "tool_created",
        "tool": {
            "name": "get_weather",
            "url": "http://api.weather.com/v1",
            "description": "Fetch weather data",
        }
    });

    let count: i64 = client
        .publish("tool_notifications", notification.to_string())
        .await
        .unwrap();

    assert_eq!(count, 1);

    let published = mock.take_published();
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].0, "tool_notifications");

    let parsed: serde_json::Value = serde_json::from_str(&published[0].1).unwrap();
    assert_eq!(parsed["event_type"], "tool_created");
    assert_eq!(parsed["tool"]["name"], "get_weather");
}
