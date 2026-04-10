use fred::mocks::Buffer;
use fred::prelude::*;
use std::sync::Arc;

#[tokio::test]
async fn test_mock_buffer_records_commands() {
    let buffer = Arc::new(Buffer::new());
    let config = Config {
        mocks: Some(buffer.clone()),
        ..Default::default()
    };
    let client = Builder::from_config(config).build().unwrap();
    client.init().await.unwrap();

    let _: String = client.set("foo", "bar", None, None, false).await.unwrap();
    let _: String = client.get("foo").await.unwrap();

    assert_eq!(buffer.len(), 2);

    let commands = buffer.take();
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].cmd, "SET");
    assert_eq!(commands[1].cmd, "GET");
}
