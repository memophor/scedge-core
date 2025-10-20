# Build stage
FROM rust:1.82-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml ./
COPY Cargo.lock ./

# Copy source code
COPY src ./src
COPY static ./static

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/scedge /usr/local/bin/scedge

# Create non-root user
RUN useradd -m -u 1000 scedge && \
    chown -R scedge:scedge /app

USER scedge

EXPOSE 8080

CMD ["scedge"]
