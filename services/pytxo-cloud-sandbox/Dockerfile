# Build from repository root (Railway: set Dockerfile path to services/pytxo-cloud-sandbox/Dockerfile)
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p pytxo-cloud-sandbox

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pytxo-cloud-sandbox /usr/local/bin/pytxo-cloud-sandbox
# Railway sets PORT; CLOUD_BIND used for local/docker-compose
ENV CLOUD_BIND=0.0.0.0:8788
CMD ["pytxo-cloud-sandbox"]
