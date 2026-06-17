# Build from repository root (Railway: set Dockerfile path to services/pytxo-link/Dockerfile)
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
# Retry crates.io fetches — Railway/mirror SSL flakes are common in Docker builds
RUN for i in 1 2 3 4 5; do cargo build --release -p pytxo-link && exit 0; echo "cargo build retry $i"; sleep 15; done; exit 1

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pytxo-link /usr/local/bin/pytxo-link
COPY services/pytxo-link/migrations /migrations
# Railway sets PORT; LINK_BIND used for local/docker-compose
ENV LINK_BIND=0.0.0.0:8787
CMD ["pytxo-link"]
