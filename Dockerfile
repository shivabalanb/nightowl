# Stage 1: Build binary
FROM rust:bookworm AS builder
WORKDIR /app

# Copy entire source tree & data
COPY . .
RUN cargo build --release --bin nightowl

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim
WORKDIR /app

# Install root CA certificates for HTTPS requests (GBFS & live APIs)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Copy pre-compiled binary and static GTFS schedule data
COPY --from=builder /app/target/release/nightowl /app/nightowl
RUN ln -s /app/nightowl /app/server
COPY data/ /app/data/

ENV PORT=3000
EXPOSE 3000

CMD ["/app/nightowl"]
