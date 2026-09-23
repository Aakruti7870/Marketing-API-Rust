# Multi-stage Dockerfile for high-performance Rust binary
FROM rust:1.88-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies & SSL libraries
RUN apt-get update && apt-get install -y pkg-config libssl-dev postgresql postgresql-client && rm -rf /var/lib/apt/lists/*

# Cache dependency builds
COPY Cargo.toml ./
RUN cargo fetch

# Copy real source files & migrations
COPY migrations ./migrations
COPY src ./src

# SQLx query! macros require a live schema during compilation.
RUN service postgresql start \
    && su postgres -c "psql -c \"CREATE DATABASE marketing_api;\"" \
    && su postgres -c "psql -d marketing_api -f /usr/src/app/migrations/0001_init.sql" \
    && su postgres -c "psql -d marketing_api -f /usr/src/app/migrations/0002_seed.sql" \
    && DATABASE_URL=postgresql://postgres@localhost/marketing_api cargo build --release --bin golde-marketing-api

# Minimal production runtime
FROM debian:bookworm-slim AS runner

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/golde-marketing-api /app/golde-marketing-api
COPY --from=builder /usr/src/app/migrations /app/migrations
COPY .env.example /app/.env.example

EXPOSE 4000

ENV PORT=4000
ENV HOST=0.0.0.0
ENV RUST_LOG=info,golde_marketing_api=info

CMD ["/app/golde-marketing-api"]
