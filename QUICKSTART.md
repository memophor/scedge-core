# Scedge Core - Quick Start Guide

Get Scedge Core running in under 5 minutes!

## Prerequisites

- Rust 1.75+ ([Install Rust](https://rustup.rs/))
- Docker (for Redis)

## Step 1: Clone and Setup

```bash
git clone https://github.com/memophor/scedge.git
cd scedge

# Copy example environment file
cp .env.example .env
```

## Step 2: Start Redis

```bash
docker run -d --name scedge-redis -p 6379:6379 redis:7-alpine
```

Or use Docker Compose:

```bash
cd examples
docker-compose up -d redis
```

## Step 3: Run Scedge Core

```bash
cargo run
```

You should see:

```
INFO scedge: Starting Scedge Core v0.1.0
INFO scedge: Redis connection established
INFO scedge: Scedge Core is running listen_addr=0.0.0.0:8080
```

## Step 4: Test the API

### Store an Artifact

```bash
curl -X POST http://localhost:8080/store \
  -H "Content-Type: application/json" \
  -d '{
    "key": "demo:greeting:en-US",
    "artifact": {
      "answer": "Hello, world!",
      "policy": {
        "tenant": "demo",
        "phi": false,
        "pii": false
      },
      "provenance": [{
        "source": "manual-test",
        "hash": "test-001"
      }],
      "hash": "greeting-v1",
      "ttl_seconds": 300
    }
  }'
```

### Lookup an Artifact

```bash
curl "http://localhost:8080/lookup?key=demo:greeting:en-US"
```

### Check Health

```bash
curl http://localhost:8080/healthz
```

### View Metrics

```bash
curl http://localhost:8080/metrics
```

## Step 5: Advanced Configuration

### Using Tenant Authentication

1. Create a `tenants.json` file:

```json
{
  "tenants": [
    {
      "tenant_id": "mycompany",
      "api_key": "secret-key-123",
      "allowed_regions": ["us-east-1"],
      "max_ttl_seconds": 86400,
      "require_phi_compliance": false,
      "require_pii_compliance": true
    }
  ]
}
```

2. Update `.env`:

```bash
SCEDGE_TENANT_KEYS_PATH=./tenants.json
```

3. Restart Scedge and use the API key:

```bash
curl -X POST http://localhost:8080/store \
  -H "Content-Type: application/json" \
  -H "X-API-Key: secret-key-123" \
  -d '{ ... }'
```

## Common Operations

### Purge by Tenant

```bash
curl -X POST http://localhost:8080/purge \
  -H "Content-Type: application/json" \
  -d '{"tenant": "demo"}'
```

### Purge by Provenance Hash

```bash
curl -X POST http://localhost:8080/purge \
  -H "Content-Type: application/json" \
  -d '{"provenance_hash": "test-001"}'
```

### Purge Specific Keys

```bash
curl -X POST http://localhost:8080/purge \
  -H "Content-Type: application/json" \
  -d '{"keys": ["demo:greeting:en-US", "demo:farewell:en-US"]}'
```

## Development Workflow

### Run with Auto-Reload

```bash
cargo install cargo-watch
cargo watch -x run
```

### Run Tests

```bash
cargo test
```

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy
```

## Docker Deployment

### Build Image

```bash
docker build -t scedge-core:latest .
```

### Run with Docker Compose

```bash
cd examples
docker-compose up
```

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `SCEDGE_PORT` | `8080` | HTTP server port |
| `SCEDGE_REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `SCEDGE_DEFAULT_TTL` | `86400` | Default TTL in seconds (24h) |
| `SCEDGE_TENANT_KEYS_PATH` | - | Path to tenants.json |
| `SCEDGE_JWT_SECRET` | - | Secret for JWT validation |
| `SCEDGE_EVENT_BUS_ENABLED` | `true` | Enable event bus |
| `SCEDGE_METRICS_ENABLED` | `true` | Enable Prometheus metrics |
| `SCEDGE_LOG_LEVEL` | `info` | Log level (error/warn/info/debug/trace) |

## Troubleshooting

### "Failed to connect to Redis"

Make sure Redis is running:

```bash
docker ps | grep redis
```

If not running, start it:

```bash
docker start scedge-redis
```

### "Port already in use"

Change the port in `.env`:

```bash
SCEDGE_PORT=9090
```

### "No tenant configurations loaded"

This is just a warning. Scedge will work without tenant auth, but API key validation will be disabled.

## Next Steps

- Read the [README](README.md) for architecture details
- Check out [CONTRIBUTING.md](CONTRIBUTING.md) to contribute
- Explore the [examples](examples/) directory
- Set up SynaGraph event integration

## Need Help?

- Open an issue: https://github.com/memophor/scedge/issues
- Read the docs: https://github.com/memophor/scedge/wiki
- Join discussions: https://github.com/memophor/scedge/discussions

Happy caching! 🚀
