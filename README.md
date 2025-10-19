# Scedge

Scedge is the Rust-based edge cache (“knowledge CDN”) for the Memophor platform. It delivers cached knowledge artifacts within tens of milliseconds while honoring provenance, policy, and invalidation signals emitted by SynaGraph.

## Features (scaffold)
- Axum HTTP service exposing `GET /lookup`, `POST /store`, and `POST /purge` endpoints.
- Artifact payloads capture provenance, policy context, confidence, TTL, and ETag metadata.
- In-memory cache with configurable TTL defaults and a background janitor that clears expired entries.
- Structured logging via `tracing` with graceful shutdown handling.

## Getting Started

### Prerequisites
- Rust toolchain (`rustup` recommended)

### Run locally
```bash
export SCEDGE_ADDR=0.0.0.0:9090
export SCEDGE_DEFAULT_TTL=300
export SCEDGE_JANITOR_SECONDS=30

cargo run
```

### API quickstart
```bash
# Store an artifact from the origin
curl -X POST http://localhost:9090/store \
  -H 'Content-Type: application/json' \
  -d '{
    "key": "tenant:intent:en",
    "artifact": {
      "content": {"plan": "triage"},
      "provenance": {"source": "knowlemesh"},
      "policy": {"region": "us-east"},
      "confidence": 0.96,
      "ttl_seconds": 120,
      "etag": "artifact-v1",
      "metadata": {"locale": "en-US"}
    }
  }'

# Lookup from the edge cache
curl "http://localhost:9090/lookup?key=tenant:intent:en"

# Purge a set of artifacts
curl -X POST http://localhost:9090/purge \
  -H 'Content-Type: application/json' \
  -d '{"keys": ["tenant:intent:en"]}'
```

### Tests
```bash
cargo check
```

## Roadmap
- Swap the in-memory store for Redis / KeyDB backed caches.
- Add ANN similarity lookups (HNSW / FAISS) to serve semantically-near artifacts.
- Wire Prometheus metrics and OpenTelemetry tracing exporters.
- Integrate policy + provenance enforcement hooks from SynaGraph events.

