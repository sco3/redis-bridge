# Redis Bridge

A Rust application that subscribes to Redis notifications and automatically creates tools in the MCP Gateway via REST API.

## Overview

This application:
1. Subscribes to a Redis Pub/Sub channel for tool notifications
2. When a notification is received, parses the JSON payload
3. Generates a JWT token for authentication (using HS256 algorithm from the mcp-benchmark project)
4. Sends a POST request to the MCP Gateway's `/tools` endpoint to create the tool

## Features

- **Redis Pub/Sub Subscription**: Listens for tool creation notifications on a configurable Redis channel
- **JWT Token Generation**: Implements HS256 JWT token generation matching the mcp-benchmark algorithm
- **REST API Integration**: Sends tool creation requests to the MCP Gateway
- **Configurable**: All parameters configurable via CLI arguments or environment variables
- **Serde Serialization**: Full JSON ser/de for ToolCreate structures

## Configuration

All parameters can be set via CLI arguments or environment variables:

| CLI Argument | Environment Variable | Default | Description |
|-------------|---------------------|---------|-------------|
| `--redis-url` | `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `--redis-channel` | `REDIS_CHANNEL` | `tool_notifications` | Redis channel to subscribe to |
| `--gateway-url` | `GATEWAY_URL` | `http://localhost:4444` | MCP Gateway base URL |
| `--tool-endpoint` | `TOOL_ENDPOINT` | `/tools` | Tool creation endpoint path |
| `--jwt-secret` | `K6_JWT_SECRET_KEY` | `my-test-key-but-now-longer-than-32-bytes` | JWT signing secret |
| `--jwt-username` | `K6_JWT_USERNAME` | `admin@example.com` | JWT subject/username |
| `--jwt-audience` | `K6_JWT_AUDIENCE` | `mcpgateway-api` | JWT audience claim |
| `--jwt-issuer` | `K6_JWT_ISSUER` | `mcpgateway` | JWT issuer claim |
| `--jwt-algorithm` | `K6_JWT_ALGORITHM` | `HS256` | JWT signing algorithm |
| `--tool-visibility` | `TOOL_VISIBILITY` | `public` | Default tool visibility |
| `--tool-integration-type` | `TOOL_INTEGRATION_TYPE` | `REST` | Default integration type |
| `--tool-request-type` | `TOOL_REQUEST_TYPE` | `POST` | Default request type |
| `--use-predefined-token` | `USE_PREDEFINED_TOKEN` | `false` | Use pre-defined bearer token |
| `--bearer-token` | `K6_BEARER_TOKEN` | *(not set)* | Pre-defined bearer token |

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
- **`redis_subscriber.rs`**: Redis Pub/Sub subscription logic
- **`api_client.rs`**: REST API client for tool creation endpoint
- **`main.rs`**: Application entry point, orchestrates all components

### Data Flow

1. Application starts and parses CLI/environment configuration
2. Connects to Redis and subscribes to the configured channel
3. Waits for messages on the channel
4. When a message arrives:
   - Parses the JSON payload into a `ToolCreate` structure
   - Generates a JWT token for authentication
   - Sends a POST request to the MCP Gateway `/tools` endpoint
   - Logs success or failure

## Expected JSON Format

The application expects JSON messages on the Redis channel in the following format:

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

# Send a test message to Redis
redis-cli PUBLISH tool_notifications '{"name":"test-tool","url":"http://localhost:8080/test","description":"Test tool","integrationType":"REST","requestType":"POST"}'
```

## License

Apache-2.0
