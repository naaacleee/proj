# Dockerfile для backend (сборка бинарника mentor-api).
# Сборка без TLS-зависимостей: HTTP внутри compose-сети,
# TLS терминируется на reverse-proxy при деплое на VPS.

FROM rust:1.83-slim AS builder
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
COPY crates ./crates
RUN cargo build --release -p mentor-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/mentor-api /usr/local/bin/mentor-api
ENV MENTOR_BIND=0.0.0.0:8080
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/healthz || exit 1
ENTRYPOINT ["mentor-api"]
