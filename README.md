# Redis Bridge

A Rust application that reads notifications from Redis Streams and automatically creates tools in the MCP Gateway via REST API.

## Overview

This application:
1. Reads messages from a Redis Stream using consumer groups
2. When a notification is received, parses the JSON payload
3. Generates a JWT token for authentication (using HS256 algorithm from the mcp-benchmark project)
4. Sends a POST request to the MCP Gateway's `/tools` endpoint to create the tool
5. Acknowledges the message in the stream after successful processing

## Features

- **Redis Streams Consumer Groups**: Reliable message processing with automatic acknowledgment
- **Message Persistence**: Messages remain in the stream until successfully processed
- **JWT Token Generation**: Implements HS256 JWT token generation matching the mcp-benchmark algorithm
- **REST API Integration**: Sends tool creation requests to the MCP Gateway
- **Configurable**: All parameters configurable via CLI arguments or environment variables
- **Serde Serialization**: Full JSON ser/de for ToolCreate structures

## Configuration

All parameters can be set via CLI arguments or environment variables:

| CLI Argument | Environment Variable | Default | Description |
|-------------|---------------------|---------|-------------|
| `--redis-url` | `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `--redis-stream` | `REDIS_STREAM` | `tool_notifications_stream` | Redis stream key to read from |
| `--redis-stream-group` | `REDIS_GROUP` | `bridge_consumers` | Consumer group name |
| `--redis-stream-consumer` | `REDIS_CONSUMER` | `bridge_consumer_1` | Consumer name (should be unique per instance) |
| `--gateway-url` | `GATEWAY_URL` | `http://localhost:8080` | MCP Gateway base URL |
| `--tool-endpoint` | `TOOL_ENDPOINT` | `/tools` | Tool creation endpoint path |
| `--jwt-secret` | `JWT_SECRET_KEY` | `my-test-key-but-now-longer-than-32-bytes` | JWT signing secret |
| `--jwt-username` | `JWT_USERNAME` | `admin@example.com` | JWT subject/username |
| `--jwt-audience` | `JWT_AUDIENCE` | `mcpgateway-api` | JWT audience claim |
| `--jwt-issuer` | `JWT_ISSUER` | `mcpgateway` | JWT issuer claim |
| `--jwt-algorithm` | `JWT_ALGORITHM` | `HS256` | JWT signing algorithm |
| `--tool-visibility` | `TOOL_VISIBILITY` | `public` | Default tool visibility |
| `--tool-integration-type` | `TOOL_INTEGRATION_TYPE` | `REST` | Default integration type |
| `--tool-request-type` | `TOOL_REQUEST_TYPE` | `POST` | Default request type |
| `--use-predefined-token` | `USE_PREDEFINED_TOKEN` | `false` | Use pre-defined bearer token |
| `--bearer-token` | `BEARER_TOKEN` | *(not set)* | Pre-defined bearer token |

## Usage

### Basic Usage

```bash
# Using defaults
cargo run

# With custom Redis URL and gateway
cargo run -- --redis-url redis://my-redis:6379 --gateway-url http://gateway:8080

# Using environment variables
REDIS_URL=redis://my-redis:6379 GATEWAY_URL=http://gateway:8080 cargo run
```

### With Custom Stream Configuration

```bash
# Custom stream and consumer group
cargo run -- \
  --redis-stream my_notifications \
  --redis-stream-group my_consumers \
  --redis-stream-consumer consumer_instance_1
```

### With JWT Configuration

```bash
cargo run -- \
  --jwt-secret "your-secret-key-here" \
  --jwt-username "admin@example.com" \
  --gateway-url "http://localhost:4444"
```

### Using Pre-defined Bearer Token

```bash
cargo run -- --use-predefined-token --bearer-token "your-token-here"
```

## Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## Architecture

### Modules

- **`jwt.rs`**: JWT token generation using HS256 (HMAC-SHA256), matching the mcp-benchmark algorithm
- **`schemas.rs`**: ToolCreate structure with full serde support for JSON serialization/deserialization
- **`config.rs`**: CLI configuration using clap with environment variable support
- **`redis_subscriber.rs`**: Redis Streams consumer logic with consumer group management
- **`api_client.rs`**: REST API client for tool creation endpoint
- **`main.rs`**: Application entry point, orchestrates all components

### Data Flow

1. Application starts and parses CLI/environment configuration
2. Connects to Redis and ensures the consumer group exists (creates it if needed)
3. Reads messages from the configured stream using `XREADGROUP`
4. When a message arrives:
   - Parses the JSON payload from the `payload` field
   - Generates a JWT token for authentication
   - Sends a POST request to the MCP Gateway `/tools` endpoint
   - Acknowledges the message with `XACK` after successful processing
   - Logs success or failure

## Expected JSON Format

The application expects messages in the Redis stream with a `payload` field containing JSON in the following format:

```json
{
  "payload": "{\"name\":\"my-tool\",\"url\":\"http://localhost:8080/tool\",\"description\":\"A sample tool\",\"integrationType\":\"REST\",\"requestType\":\"POST\",\"inputSchema\":{\"type\":\"object\",\"properties\":{}},\"visibility\":\"public\"}"
}
```

Or the JSON can be directly in the payload field value:

```json
{
  "name": "my-tool",
  "url": "http://localhost:8080/tool",
  "description": "A sample tool",
  "integrationType": "REST",
  "requestType": "POST",
  "inputSchema": {
    "type": "object",
    "properties": {}
  },
  "visibility": "public"
}
```

All fields from the MCP Gateway's `ToolCreate` schema are supported.

## Adding Messages to the Stream

To add a test notification to the Redis stream:

```bash
# Using redis-cli
redis-cli XADD tool_notifications_stream '*' payload '{"name":"test-tool","url":"http://localhost:8080/test","description":"Test tool","integrationType":"REST","requestType":"POST"}'
```

## Redis Streams vs Pub/Sub

This application uses **Redis Streams** instead of Pub/Sub for the following advantages:

- **Message Persistence**: Messages remain in the stream until explicitly deleted or acknowledged
- **Reliable Processing**: Messages are only acknowledged after successful processing
- **Consumer Groups**: Multiple instances can share the workload efficiently
- **Replay Capability**: Can reprocess messages if needed (by changing the stream ID)
- **No Message Loss**: Unlike Pub/Sub, messages aren't lost if the consumer is temporarily unavailable

## JWT Token Structure

The generated JWT tokens follow this structure (matching mcp-benchmark):

**Header:**
```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```

**Claims:**
```json
{
  "sub": "admin@example.com",
  "exp": <1 year from now>,
  "iat": <current time>,
  "aud": "mcpgateway-api",
  "iss": "mcpgateway",
  "jti": "<random 16-char hex>",
  "token_use": "session",
  "user": {
    "email": "admin@example.com",
    "full_name": "Rust MCP Bridge",
    "is_admin": true,
    "auth_provider": "local"
  }
}
```

## Testing

```bash
# Run unit tests
cargo test

# Run linting (clippy with pedantic lints)
just lint

# Run end-to-end smoke test (requires running Redis + MCP Gateway)
cargo run --bin smoke-test

# Run smoke test with custom configuration
REDIS_URL=redis://my-redis:6379 GATEWAY_URL=http://myhost:8080 cargo run --bin smoke-test
```

## Development

This project uses [Just](https://just.systems/) for common development tasks:

```bash
# List available targets
just

# Build in release mode
just build

# Run tests
just test

# Run linter (clippy)
just lint

# Format code
just fmt

# Full CI-style check (fmt, check, clippy, test)
just ci
```

## License

Apache-2.0
