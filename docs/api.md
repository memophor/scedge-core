# Scedge Core API Documentation

Complete API reference for Scedge Core v0.1

## Base URL

```
http://localhost:8090
```

Configure the port via `SCEDGE_PORT` environment variable.

---

## Authentication

Currently, Scedge Core v0.1 runs in open mode for development. Future versions will support:
- API Key authentication
- JWT token validation
- Tenant-based access control

---

## Endpoints

### Health Check

Check if the service is running and healthy.

**Endpoint:** `GET /healthz` or `GET /health`

**Response:**
```json
{
  "service": "scedge-core",
  "status": "healthy",
  "version": "0.1.0"
}
```

**Status Codes:**
- `200 OK` - Service is healthy

---

### Prometheus Metrics

Retrieve Prometheus-compatible metrics.

**Endpoint:** `GET /metrics`

**Response:** Plain text Prometheus format

**Example Metrics:**
```
# HELP scedge_cache_hits_total Total number of cache hits
# TYPE scedge_cache_hits_total counter
scedge_cache_hits_total 42

# HELP scedge_cache_misses_total Total number of cache misses
# TYPE scedge_cache_misses_total counter
scedge_cache_misses_total 5

# HELP scedge_cache_stores_total Total number of cache stores
# TYPE scedge_cache_stores_total counter
scedge_cache_stores_total 15
```

**Available Metrics:**
- `scedge_cache_hits_total` - Cache hit count
- `scedge_cache_misses_total` - Cache miss count
- `scedge_cache_stores_total` - Successful store operations
- `scedge_cache_purges_total` - Purge operations
- `scedge_artifacts_stored_total` - Total artifacts stored
- `scedge_artifacts_expired_total` - Expired artifacts
- `scedge_cache_size` - Current cache size (gauge)

---

### Store Artifact

Store a knowledge artifact in the cache.

**Endpoint:** `POST /store`

**Request Body:**
```json
{
  "key": "string",
  "artifact": {
    "answer": "any-json-value",
    "policy": {
      "tenant": "string",
      "phi": boolean,
      "pii": boolean,
      "region": "string (optional)",
      "compliance_tags": ["string"] (optional)
    },
    "provenance": [
      {
        "source": "string",
        "hash": "string (optional)",
        "version": "string (optional)",
        "generated_at": "ISO-8601-timestamp (optional)"
      }
    ],
    "metrics": {
      "score": number,
      "generated_at": "ISO-8601-timestamp (optional)"
    } (optional),
    "ttl_seconds": number (optional),
    "hash": "string",
    "metadata": {} (optional)
  }
}
```

**Response:**
```json
{
  "key": "demo:greeting:en-US",
  "status": "created",
  "hash": "v1",
  "expires_at": "2025-10-20T23:52:40.721571Z"
}
```

**Status Codes:**
- `200 OK` - Artifact stored successfully
- `400 Bad Request` - Invalid request format
- `500 Internal Server Error` - Server error

**Example:**
```bash
curl -X POST http://localhost:8090/store \
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
      "provenance": [{"source": "manual-test"}],
      "hash": "v1"
    }
  }'
```

---

### Lookup Artifact

Retrieve a cached artifact by key.

**Endpoint:** `GET /lookup`

**Query Parameters:**
- `key` (required) - The cache key to lookup
- `tenant` (optional) - Tenant ID for multi-tenant filtering

**Response (Success - Cache Hit):**
```json
{
  "key": "demo:greeting:en-US",
  "artifact": {
    "answer": "Hello, world!",
    "policy": {
      "tenant": "demo",
      "phi": false,
      "pii": false,
      "region": null,
      "compliance_tags": []
    },
    "provenance": [
      {
        "source": "manual-test",
        "hash": null,
        "version": null,
        "generated_at": null
      }
    ],
    "metrics": null,
    "ttl_seconds": null,
    "hash": "v1",
    "metadata": null
  },
  "expires_at": "2025-10-20T23:52:40.721571Z",
  "ttl_remaining_seconds": 86395
}
```

**Response (Cache Miss):**
```json
{
  "error": "Artifact not found"
}
```

**Status Codes:**
- `200 OK` - Artifact found
- `404 Not Found` - Artifact not in cache
- `400 Bad Request` - Missing or invalid key parameter

**Example:**
```bash
curl "http://localhost:8090/lookup?key=demo:greeting:en-US"
```

---

### Purge Artifacts

Remove one or more artifacts from the cache.

**Endpoint:** `POST /purge`

**Request Body (By Keys):**
```json
{
  "keys": ["key1", "key2", "key3"]
}
```

**Request Body (By Tenant):**
```json
{
  "tenant": "tenant-id"
}
```

**Request Body (By Provenance Hash):**
```json
{
  "provenance_hash": "hash-value"
}
```

**Response:**
```json
{
  "purged": 3
}
```

**Status Codes:**
- `200 OK` - Purge operation completed
- `400 Bad Request` - Invalid request format
- `500 Internal Server Error` - Server error

**Examples:**

Purge specific keys:
```bash
curl -X POST http://localhost:8090/purge \
  -H "Content-Type: application/json" \
  -d '{"keys": ["demo:greeting:en-US", "demo:farewell:en-US"]}'
```

Purge by tenant:
```bash
curl -X POST http://localhost:8090/purge \
  -H "Content-Type: application/json" \
  -d '{"tenant": "demo"}'
```

---

## Data Models

### CacheKey Format

Recommended semantic key structure:
```
{tenant}:{intent}:{locale}:{version}
```

**Examples:**
- `acme:greeting:en-US:v1`
- `healthcare:patient-summary:en-US`
- `api:users-list:v2`

### ArtifactPayload

The core data structure for cached knowledge:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `answer` | Any JSON | Yes | The actual knowledge/content to cache |
| `policy` | PolicyContext | Yes | Access control and compliance metadata |
| `provenance` | Array<ProvenanceInfo> | Yes | Source tracking information |
| `metrics` | ArtifactMetrics | No | Quality and confidence scores |
| `ttl_seconds` | Number | No | Time-to-live override (default: 86400) |
| `hash` | String | Yes | Version/ETag for the artifact |
| `metadata` | Object | No | Additional arbitrary metadata |

### PolicyContext

Access control and compliance metadata:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | String | Yes | Tenant/organization identifier |
| `phi` | Boolean | Yes | Contains Protected Health Information |
| `pii` | Boolean | Yes | Contains Personally Identifiable Information |
| `region` | String | No | Geographic region constraint |
| `compliance_tags` | Array<String> | No | Compliance standards (e.g., ["HIPAA", "GDPR"]) |

### ProvenanceInfo

Tracks the source and lineage of knowledge:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `source` | String | Yes | Origin of the knowledge (e.g., "gpt-4", "database") |
| `hash` | String | No | Hash of the source data |
| `version` | String | No | Version of the source |
| `generated_at` | ISO-8601 | No | When the knowledge was generated |

### ArtifactMetrics

Quality and confidence metadata:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `score` | Number | No | Confidence score (0.0-1.0, default: 1.0) |
| `generated_at` | ISO-8601 | No | Metrics timestamp |
| Additional fields | Any | No | Custom metrics via `flatten` |

---

## Response Status Codes

| Code | Meaning |
|------|---------|
| `200 OK` | Request successful |
| `400 Bad Request` | Invalid request format or parameters |
| `404 Not Found` | Resource not found (cache miss) |
| `500 Internal Server Error` | Server-side error |

---

## Rate Limiting

Currently no rate limiting in v0.1. Future versions will support configurable rate limits per tenant.

---

## Error Responses

Standard error format:
```json
{
  "error": "Error message description"
}
```

---

## Best Practices

### Key Design

1. **Use semantic keys** - Include tenant, intent, and context
2. **Version your keys** - Append version numbers when structure changes
3. **Keep keys readable** - Use colons as separators

### TTL Management

1. **Set appropriate TTLs** - Balance freshness vs. cache efficiency
2. **Use different TTLs for different data types**:
   - Static content: 7+ days
   - Dynamic data: 1-24 hours
   - Real-time data: minutes

### Policy Enforcement

1. **Always set `tenant`** - Enable multi-tenancy from day one
2. **Mark PHI/PII correctly** - Ensure compliance tracking
3. **Use compliance tags** - Track regulatory requirements

### Provenance

1. **Track sources** - Always provide the origin
2. **Version sources** - Include model/system versions
3. **Include timestamps** - Helps with debugging and auditing

---

## WebSocket Support

WebSocket support for real-time cache updates is planned for v0.2.

---

## Bulk Operations

### Bulk Store

Store multiple artifacts in sequence:

```bash
# See Testing Dashboard -> Bulk Operations
POST /store (multiple requests)
```

### Bulk Lookup

Retrieve multiple artifacts:

```bash
# See Testing Dashboard -> Bulk Operations
GET /lookup (multiple requests with different keys)
```

**Note:** Native bulk endpoints planned for v0.2.

---

## Performance Considerations

- **Cache key lookup:** Sub-5ms average
- **Store operation:** 5-15ms average
- **Purge operation:** Varies by number of keys
- **Metrics endpoint:** ~10ms

Actual performance depends on:
- Redis latency
- Network conditions
- Payload size
- Concurrent requests

---

## Testing Dashboard

Access the interactive testing dashboard at:
```
http://localhost:8090/
```

Features:
- Live metrics monitoring
- Request/response visualization
- Performance timing
- History tracking
- Example templates
- Dark mode

---

## Future API Changes (v0.2+)

Planned enhancements:
- Batch endpoints (`/batch/store`, `/batch/lookup`)
- Pattern-based purging (`/purge?pattern=tenant:*`)
- Semantic search (`/search?query=...&embedding=[...]`)
- WebSocket streaming (`/ws`)
- GraphQL API
- gRPC support

---

## Support

- **Issues:** [GitHub Issues](https://github.com/memophor/scedge-core/issues)
- **Discussions:** [GitHub Discussions](https://github.com/memophor/scedge-core/discussions)
- **Discord:** [Join our community](https://discord.gg/memophor)

---

**Last Updated:** October 2025
**API Version:** 0.1.0
**Status:** Development / Alpha
