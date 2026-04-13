# >=1.85: транзитивные зависимости (например toml_edit через sea-orm/sqlx) могут требовать поддержку edition2024 в Cargo.
FROM rust:1-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/service_processes /usr/local/bin/service_processes

EXPOSE 8080
CMD ["service_processes"]
