FROM rust:1.92-slim as builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/asset_tracker /usr/local/bin/asset_tracker

EXPOSE 8080
CMD ["asset_tracker"]
