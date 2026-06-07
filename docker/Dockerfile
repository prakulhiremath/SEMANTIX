# Multi-stage build for SEMANTIX
FROM rust:1.75 AS builder

WORKDIR /build

# Copy manifest
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release 2>&1 | head -50

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    postgresql-client \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binaries
COPY --from=builder /build/target/release/semantix-daemon /app/
COPY --from=builder /build/target/release/cost-profiler /app/
COPY --from=builder /build/target/release/benchmark /app/

# Copy configuration
COPY semantix.toml /app/
COPY schema /app/schema/

# Expose ports
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ./semantix-daemon --health-check || exit 1

# Default command
CMD ["./semantix-daemon"]
