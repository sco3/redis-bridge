use fred::mocks::{MockCommand, Mocks};
use fred::prelude::*;
use std::sync::Arc;

/// Custom mock implementing a realistic stream-notification scenario
#[derive(Debug)]
struct StreamNotificationMock {
    add_buffer: std::sync::Mutex<Vec<(String, Vec<(String, String)>)>>,
}

impl StreamNotificationMock {
    fn new() -> Self {
        Self {
            add_buffer: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn take_added(&self) -> Vec<(String, Vec<(String, String)>)> {
        self.add_buffer.lock().unwrap().drain(..).collect()
    }
}

impl Mocks for StreamNotificationMock {
    fn process_command(&self, command: MockCommand) -> Result<Value, Error> {
        if &*command.cmd == "XADD" {
            let stream = match command.args.first() {
                Some(Value::String(s)) => s.to_string(),
                Some(Value::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
                _ => {
                    return Err(Error::new(ErrorKind::InvalidArgument, "Invalid stream"));
                }
            };
            let mut fields = Vec::new();
            // Args are: stream, id, field_name, field_value
            // Skip first 2 args to get to the fields
            for arg in command.args.iter().skip(2) {
                match arg {
                    Value::String(s) => {
                        fields.push(s.to_string());
                    }
                    Value::Bytes(b) => {
                        fields.push(String::from_utf8_lossy(b).to_string());
                    }
                    _ => {}
                }
            }
            // Pair them up
            let mut pairs = Vec::new();
            let mut i = 0;
            while i + 1 < fields.len() {
                pairs.push((fields[i].clone(), fields[i + 1].clone()));
                i += 2;
            }
            self.add_buffer
                .lock()
                .unwrap()
                .push((stream, pairs));
            // Return a fake stream ID
            Ok(Value::from_static_str("1234567890123-0"))
        } else if &*command.cmd == "XGROUP" {
            Ok(Value::from_static_str("OK"))
        } else if &*command.cmd == "XREADGROUP" {
            Ok(Value::Null)
        } else if &*command.cmd == "XACK" {
            Ok(Value::Integer(1))
        } else {
            Err(Error::new(ErrorKind::Unknown, "Unimplemented."))
        }
    }
}

#[tokio::test]
async fn test_mock_custom_stream_notification() {
    let mock = Arc::new(StreamNotificationMock::new());
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

    let result: String = client
        .xadd(
            "tool_notifications_stream",
            false,
            None::<()>,
            "*",
            vec![("payload", notification.to_string())],
        )
        .await
        .unwrap();

    // Verify we got a stream ID back
    assert!(!result.is_empty());

    let added = mock.take_added();
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].0, "tool_notifications_stream");

    // Debug: print what we got
    eprintln!("Added: {:?}", added[0]);

    // Find the payload field
    let payload = added[0]
        .1
        .iter()
        .find(|(k, _)| k == "payload")
        .map(|(_, v)| v)
        .expect("payload field not found");

    let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
    assert_eq!(parsed["event_type"], "tool_created");
    assert_eq!(parsed["tool"]["name"], "get_weather");
}
