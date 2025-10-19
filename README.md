<div align="center">

# ⚡ Scedge Core

**Smart Cache on the Edge**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Redis](https://img.shields.io/badge/redis-7%2B-red.svg)](https://redis.io)

*Edge-layer cache for AI memory and knowledge delivery*

[Getting Started](#-getting-started) •
[Documentation](#-documentation) •
[Architecture](#-architecture) •
[Contributing](#-contributing) •
[Community](#-community)

</div>

---

## Overview

**Scedge Core** is an open-source, policy-aware edge cache built for distributed AI systems. It stores and serves **knowledge artifacts**—not just static data—providing instant, low-latency responses to repeated AI queries while reducing GPU and compute costs by up to 90%.

### Key Features

| Feature | Description |
|---------|-------------|
| 🚀 **Sub-50ms Latency** | Lightning-fast artifact retrieval at the edge |
| 🔐 **Policy-Aware** | Multi-tenant with PHI/PII compliance built-in |
| 🎯 **Semantic Keys** | Cache by meaning: intent, tenant, locale, version |
| 🔄 **Graph-Aware** | Intelligent invalidation via SynaGraph events |
| 📊 **Observable** | Prometheus metrics and structured logging |
| 🔌 **Pluggable** | Trait-based architecture for custom backends |

### Part of the Memophor Knowledge Mesh

Scedge Core is the **edge layer** of the Memophor platform:

| Component | Role |
|------------|------|
| **[SynaGraph](https://github.com/memophor/synagraph)** | Graph + vector + temporal knowledge engine |
| **[Knowlemesh](https://github.com/memophor/knowlemesh)** | Orchestration and governance control plane |
| **[SeTGIN](https://github.com/memophor/setgin)** | Self-tuning intelligence network |
| **Scedge Core** | Smart edge cache — *this repository* |

---

## 🚀 Getting Started

### Quick Start (5 minutes)

```bash
# 1. Start Redis
docker run -d --name redis -p 6379:6379 redis:7

# 2. Clone and run Scedge
git clone https://github.com/memophor/scedge-core.git
cd scedge-core
cargo run

# 3. Test the API
curl -X POST http://localhost:8080/store \
  -H "Content-Type: application/json" \
  -d '{
    "key": "demo:greeting:en-US",
    "artifact": {
      "answer": "Hello, world!",
      "policy": {"tenant": "demo", "phi": false, "pii": false},
      "provenance": [{"source": "manual-test"}],
      "hash": "v1"
    }
  }'

curl "http://localhost:8080/lookup?key=demo:greeting:en-US"
```

**📖 See [QUICKSTART.md](QUICKSTART.md) for detailed instructions**

### Prerequisites

- **Rust 1.75+** ([Install](https://rustup.rs/))
- **Redis 7+** (via Docker or native install)
- **Docker** (optional, for containerized deployment)

---

## 📚 Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - Get running in 5 minutes
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Technical deep-dive
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
- **[VISION.md](docs/VISION.md)** - Project vision and roadmap
- **[API Reference](docs/api.md)** - Complete API documentation

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│     Client / AI Agent / Application     │
└───────────────┬─────────────────────────┘
                │ HTTP Request
                ▼
┌─────────────────────────────────────────┐
│          Scedge Core PoP                │
│  ┌─────────────────────────────────┐   │
│  │  Policy Enforcement              │   │
│  │  • Tenant validation             │   │
│  │  • JWT / API key auth           │   │
│  │  • PHI/PII compliance           │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Redis Cache Backend             │   │
│  │  • Semantic key lookup          │   │
│  │  • TTL management               │   │
│  │  • Pattern-based scan           │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Event Bus (Redis Pub/Sub)      │   │
│  │  • Graph invalidation events    │   │
│  │  • Provenance-based purging     │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
                │
                │ Metrics
                ▼
          [Prometheus]
```

### Core Components

- **Cache Backend** - Pluggable trait-based architecture (Redis, SQLite, RocksDB)
- **Policy Engine** - Multi-tenant auth with compliance enforcement
- **Event Bus** - Graph-aware cache invalidation via Pub/Sub
- **Metrics** - Prometheus-compatible observability
- **REST API** - `/lookup`, `/store`, `/purge`, `/healthz`, `/metrics`

---

## 📦 Installation

### From Source

```bash
git clone https://github.com/memophor/scedge-core.git
cd scedge-core
cargo build --release
./target/release/scedge
```

### With Docker

```bash
docker build -t scedge-core:latest .
docker run -p 8080:8080 -e SCEDGE_REDIS_URL=redis://redis:6379 scedge-core:latest
```

### With Docker Compose

```bash
cd examples
docker-compose up
```

---

## 🔧 Configuration

Configure via environment variables (see [.env.example](.env.example)):

| Variable | Default | Description |
|----------|---------|-------------|
| `SCEDGE_PORT` | `8080` | HTTP server port |
| `SCEDGE_REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `SCEDGE_DEFAULT_TTL` | `86400` | Default TTL in seconds |
| `SCEDGE_TENANT_KEYS_PATH` | - | Path to tenant configuration JSON |
| `SCEDGE_JWT_SECRET` | - | Secret for JWT validation |
| `SCEDGE_EVENT_BUS_ENABLED` | `true` | Enable event bus |
| `SCEDGE_METRICS_ENABLED` | `true` | Enable Prometheus metrics |

---

## 🌐 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/healthz` | Health check |
| `GET` | `/metrics` | Prometheus metrics |
| `GET` | `/lookup?key=...` | Retrieve cached artifact |
| `POST` | `/store` | Store new artifact |
| `POST` | `/purge` | Invalidate artifacts |

**📖 See [API Documentation](docs/api.md) for request/response schemas**

---

## 🧑‍💻 Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo fmt        # Format code
cargo clippy     # Lint
cargo audit      # Security audit
```

### Local Development

```bash
# Start dependencies
docker-compose -f examples/docker-compose.yml up redis

# Run with hot-reload
cargo watch -x run
```

---

## 🤝 Contributing

We welcome contributions! Please see:

- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines
- **[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** - Community standards
- **[Good First Issues](https://github.com/memophor/scedge-core/labels/good-first-issue)** - Great starting points

### How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and add tests
4. Run `cargo fmt && cargo clippy`
5. Commit with signed commits (`git commit -S -m "feat: add amazing feature"`)
6. Push and create a Pull Request

---

## 🗺️ Roadmap

| Milestone | Status | Target |
|-----------|--------|--------|
| **v0.1** - Redis backend, core APIs, metrics | 🚧 In Progress | Q4 2025 |
| **v0.2** - ANN semantic search, event bus | 🧱 Planned | Q1 2026 |
| **v0.3** - Policy middleware, WASM plugins | 🧱 Planned | Q2 2026 |
| **v1.0** - Production-ready, stable API | ⏳ Planned | Q3 2026 |

**📖 See [VISION.md](docs/VISION.md) for long-term roadmap**

---

## 📊 Performance

Scedge Core is designed for speed:

- **Sub-50ms** artifact retrieval
- **10,000+ RPS** on commodity hardware
- **90% reduction** in GPU compute for cached queries
- **Horizontal scaling** via Redis clustering

*Benchmarks coming in v0.1*

---

## 🔒 Security

- **Multi-tenant isolation** via policy enforcement
- **JWT + API key** authentication
- **PHI/PII compliance** tagging
- **Signed commits** required for contributions

**📖 See [SECURITY.md](SECURITY.md) for reporting vulnerabilities**

---

## 📜 License

Copyright © 2025 Memophor Labs

Licensed under the **Apache License, Version 2.0**.
See [LICENSE](LICENSE) for details.

---

## 🌟 Community

- **GitHub Discussions** - [Join the conversation](https://github.com/memophor/scedge-core/discussions)
- **Issues** - [Report bugs or request features](https://github.com/memophor/scedge-core/issues)
- **Twitter** - [@memophor](https://twitter.com/memophor)
- **Discord** - [Join our community](https://discord.gg/memophor)

---

## 🙏 Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Blazing fast and memory safe
- [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework
- [Redis](https://redis.io/) - In-memory data structure store
- [Prometheus](https://prometheus.io/) - Monitoring and alerting

---

<div align="center">

**⚡ Move knowledge, not tokens.**

Made with ❤️ by [Memophor Labs](https://memophor.com)

</div>
