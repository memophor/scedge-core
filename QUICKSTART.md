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

## Run with SynaGraph

Use the dedicated compose file when you want Scedge Core to share the `memonet` network with an existing [SynaGraph](https://github.com/memophor/synagraph) stack.

1. Make sure the shared Docker network exists:

   ```bash
   docker network inspect memonet >/dev/null 2>&1 || docker network create memonet
   ```

2. Start or verify your SynaGraph services (they must attach to the same `memonet` network).

3. From the Scedge repository root, start the Scedge services:

   ```bash
   docker compose -f docker-compose.scedge.yml up -d
   ```

   The compose file builds Scedge from the local checkout (`build: .`). Update the path if you run the compose file from a different directory.

4. Confirm the stack is healthy:

   ```bash
   curl -s http://localhost:8090/healthz | jq .
   ```

5. Exercise the cache (Scedge now hydrates cold keys from SynaGraph automatically):

   ```bash
   # First lookup hydrates from SynaGraph and caches the capsule
   curl -s "http://localhost:8090/lookup?tenant=acme&key=acme:analytics:report" | jq .

   # Second lookup is a HIT from Redis (<50ms)
   curl -s "http://localhost:8090/lookup?tenant=acme&key=acme:analytics:report" | jq .

   # Optional: seed manually if your SynaGraph instance has no data yet
   curl -s -X POST http://localhost:8090/store \
     -H "content-type: application/json" \
     -d '{
       "key": "acme:analytics:report",
       "artifact": {
         "answer": "Quarterly revenue was up 23%.",
         "policy": {"tenant": "acme", "phi": false, "pii": false},
         "provenance": [{"source": "syna:demo", "hash": "demo-1"}],
         "hash": "demo-1",
         "ttl_seconds": 3600
       }
     }' | jq .

   # Optional: purge and watch the next lookup hydrate again
   curl -s -X POST http://localhost:8090/purge \
     -H "content-type: application/json" \
     -d '{"tenant":"acme", "key":"acme:analytics:report"}' | jq .
   ```

6. Open the UI at http://localhost:8090 to watch cache hit metrics update while you test.

With the event bus enabled, updates emitted on the `synagraph.cache` NATS subject flow straight into Scedge and invalidate cached capsules in real time.


## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `SCEDGE_PORT` | `8080` | HTTP server port |
| `SCEDGE_REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `SCEDGE_UPSTREAM_URL` | - | Base URL for SynaGraph lookups |
| `SCEDGE_UPSTREAM_TIMEOUT_SECS` | `5` | Timeout in seconds for upstream calls |
| `SCEDGE_DEFAULT_TTL` | `86400` | Default TTL in seconds (24h) |
| `SCEDGE_TENANT_KEYS_PATH` | - | Path to tenants.json |
| `SCEDGE_JWT_SECRET` | - | Secret for JWT validation |
| `SCEDGE_EVENT_BUS_ENABLED` | `true` | Enable event bus |
| `SCEDGE_EVENT_BUS_URL` | `nats://127.0.0.1:4222` | NATS server for graph invalidation events |
| `SCEDGE_EVENT_BUS_CHANNEL` | `synagraph.cache` | NATS subject for invalidation events |
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
