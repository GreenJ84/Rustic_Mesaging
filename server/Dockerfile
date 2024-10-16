FROM rust:1.80 as builder
LABEL authors="jessegreenough"
WORKDIR /build
COPY . .
RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/server ./server

FROM ubuntu:22.04
WORKDIR /app

## copy the main binary
COPY --from=builder /build/server ./
COPY --from=builder /build/Rocket.toml ./static
#COPY --from=builder /build/static ./static
#COPY --from=builder /build/templates ./templates

## ensure the container listens globally on port 8080
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080

CMD ./server