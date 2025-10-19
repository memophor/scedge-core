# ⚡ Scedge Core — Smart Cache on the Edge
> *Edge-layer cache for AI memory and knowledge delivery.*

---

## Overview

**Scedge Core** is the open-source foundation of the Scedge platform — a **semantic, policy-aware edge cache** built for distributed AI systems.
It stores and serves **knowledge artifacts**, not static files, providing instant, low-latency responses to repeated AI queries while reducing GPU and compute usage by up to 90%.

Scedge Core forms the **edge layer** of the [Memophor Knowledge Mesh](https://github.com/memophor), alongside:

| Component | Role |
|------------|------|
| **SynaGraph** | Graph + vector + temporal knowledge engine |
| **Knowlemesh** | Orchestration and governance control plane |
| **SeTGIN** | Self-tuning intelligence network that learns from telemetry |
| **Scedge** | Smart Cache on the Edge — *this repository* |

---

## ✨ Features

| Capability | Description |
|-------------|-------------|
| **Redis-based caching** | Pluggable cache backend (Redis, KeyDB, or SQLite). |
| **Semantic keys** | Cache entries keyed by *meaning* — intent, tenant, policy, locale, and version. |
| **Policy enforcement** | Tenant and compliance tags (HIPAA/GDPR-ready). |
| **Graph-aware invalidation** | Listens for `SUPERSEDED_BY` and `REVOKE_CAPSULE` events from SynaGraph. |
| **Fast APIs** | `/lookup`, `/store`, and `/purge` endpoints for artifact lifecycle. |
| **Lightweight & portable** | Single Rust binary; deploy anywhere. |
| **Observability** | Prometheus metrics, `/healthz`, structured logs. |

---

## 🧱 Architecture

```
Client / Agent
   │
   │ 1️⃣  Request or query
   ▼
┌────────────────────────────────────────────┐
│  Scedge Core PoP (Rust microservice)      │
│  • Semantic cache (Redis / SQLite)        │
│  • /lookup  /store  /purge APIs           │
│  • Tenant policy enforcement              │
│  • TTL + provenance awareness             │
└────────────────────────────────────────────┘
   │
   │ 2️⃣  Cache miss → forward to origin
   ▼
┌────────────────────────────────────────────┐
│  Knowlemesh / SynaGraph Origin             │
│  • Generates / validates knowledge         │
│  • Publishes graph events to PoPs          │
└────────────────────────────────────────────┘
```

---

## ⚙️ Quick Start

### 1️⃣ Prerequisites
- Rust 1.75+ (nightly or stable)
- Redis 7+
- Docker (optional for local dev)

### 2️⃣ Run locally

```bash
git clone https://github.com/memophor/scedge.git
cd scedge

# Start Redis
docker run -d --name redis -p 6379:6379 redis:7

# Run Scedge Core
cargo run
```

### 3️⃣ Example requests

```bash
# Store an artifact
curl -X POST http://localhost:8080/store \
  -H "Content-Type: application/json" \
  -d '{
    "key":"reset_password:mobile:en-US",
    "tenant":"acme",
    "artifact":{
      "answer":"Reset link sent",
      "ttl_sec":86400
    }
  }'

# Lookup
curl "http://localhost:8080/lookup?key=reset_password:mobile:en-US&tenant=acme"

# Purge
curl -X POST http://localhost:8080/purge \
  -H "Content-Type: application/json" \
  -d '{"tenant":"acme","provenance_hash":"sha256:xyz"}'
```

---

## 🧩 Configuration

| Env Var | Description | Default |
|----------|--------------|---------|
| `SCEDGE_PORT` | Port to bind | `8080` |
| `SCEDGE_REDIS_URL` | Redis connection URI | `redis://127.0.0.1:6379` |
| `SCEDGE_DEFAULT_TTL` | Default cache TTL (seconds) | `86400` |
| `SCEDGE_LOG_LEVEL` | `info`, `debug`, etc. | `info` |
| `SCEDGE_TENANT_KEYS_PATH` | JSON file of tenant API keys | `./tenants.json` |

---

## 📦 Artifact Schema

```json
{
  "answer": "string or template",
  "policy": {"tenant":"acme","phi":true},
  "provenance": [{"source":"doc://handbook#42"}],
  "metrics": {"_score":0.91,"generated_at":"2025-10-18"},
  "ttl_sec": 259200,
  "hash": "etag-abc123"
}
```

Artifacts are signed at origin and validated at the edge.

---

## 🔌 Integration Points

| Source | Event | Action |
|--------|--------|--------|
| **SynaGraph** | `SUPERSEDED_BY` | Purge cached artifacts sharing the old provenance hash. |
| **Knowlemesh** | Policy update | Invalidate affected tenants' keys. |
| **SeTGIN** | PerfPolicy adjustment | Update TTL or cache thresholds. |

---

## 🧠 Why Open Source?

- **Developer adoption:** Make caching and knowledge reuse easy for any AI system.
- **Transparency:** Open design for policy enforcement and provenance.
- **Extensibility:** Community-driven backends (Redis, KeyDB, RocksDB).
- **Innovation loop:** External contributors can extend edge learning and analytics.

> Scedge Core follows the [Apache 2.0 License](LICENSE). Commercial orchestration and analytics remain part of **Knowlemesh Cloud**.

---

## 🧩 Repository Structure

```
scedge/
 ├── src/
 │   ├── main.rs              # PoP entrypoint
 │   ├── api.rs               # /lookup /store /purge handlers
 │   ├── cache.rs             # Redis adapter + trait
 │   ├── policy.rs            # JWT & policy enforcement
 │   ├── metrics.rs           # Prometheus integration
 │   └── events.rs            # Redis pub/sub or NATS hooks
 ├── examples/
 │   └── docker-compose.yml   # Local Redis + Scedge
 ├── tests/
 ├── Cargo.toml
 └── LICENSE
```

---

## 🧰 Roadmap

| Milestone | Description | Status |
|------------|-------------|---------|
| **v0.1** | Redis cache backend, `/lookup` `/store` `/purge`, Prometheus metrics | 🚧 In progress |
| **v0.2** | ANN near-duplicate search, provenance purge events | 🧱 Planned |
| **v0.3** | Policy enforcement middleware (JWT + WASM) | 🧱 Planned |
| **v1.0** | Stable API + CLI; integration with Knowlemesh + SynaGraph | ⏳ Target Q2 2026 |

---

## 🧑‍🤝‍🧑 Contributing

We welcome issues, discussions, and PRs.

1. Fork the repo and run `cargo fmt && cargo clippy` before committing.
2. Write integration tests where possible (`cargo test`).
3. Update docs for new APIs or config flags.
4. Sign your commits (`git commit -S`).

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## 🔒 License

**Apache 2.0** — see [LICENSE](LICENSE).
Copyright © 2025 Memophor Labs.

---

### ✨ Summary

Scedge Core is the open-source **Knowledge CDN** that brings AI memory to the edge.
It's fast, policy-aware, and built for the Federated Knowledge Mesh.
Deploy it anywhere — cloud, on-prem, or your laptop — and give your AI the power to remember.

> *"Move knowledge, not tokens."*
