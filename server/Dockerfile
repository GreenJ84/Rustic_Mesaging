FROM rust:1.80 as builder
LABEL authors="jessegreenough"
WORKDIR /build

COPY . .

ARG SERVER_URL
ENV SERVER_URL=${SERVER_URL}
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/server ./server

FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y curl libpq-dev libpq5 postgresql-client build-essential && \
    rm -rf /var/lib/apt/lists/* && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /app

## copy the main binary
COPY --from=builder /build/server ./
COPY --from=builder /build/Cargo.toml ./
COPY --from=builder /build/Rocket.toml ./
COPY --from=builder /build/diesel.toml ./
COPY --from=builder /build/migrations ./migrations

## ensure the container listens globally on port 8080
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
EXPOSE 8000


CMD ./server

