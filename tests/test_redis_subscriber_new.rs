use clap::Parser;
use redis_bridge::config::Config;
use redis_bridge::redis_subscriber::RedisSubscriber;

#[test]
fn test_redis_subscriber_new() {
    let config = Config::try_parse_from(["redis-bridge"]).unwrap();
    let subscriber = RedisSubscriber::new(config.clone());
    assert_eq!(subscriber.redis_url(), config.redis_url);
    assert_eq!(subscriber.redis_channel(), config.redis_channel);
}
