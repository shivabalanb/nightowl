# Stage 1: Build binary with cached dependencies
FROM rust:1.85-bookworm AS builder
WORKDIR /app

# Cache dependency layer
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin src/ui && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/server.rs && \
    echo "pub fn lib() {}" > src/lib.rs && \
    touch src/ui/index.html && \
    cargo build --release || true

# Copy entire source tree & data
COPY . .
RUN cargo build --release --bin server

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim
WORKDIR /app

# Install root CA certificates for HTTPS requests (GBFS & live APIs)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Copy pre-compiled binary and static GTFS schedule data
COPY --from=builder /app/target/release/server /app/server
COPY data/ /app/data/

ENV PORT=3000
EXPOSE 3000

CMD ["/app/server"]
