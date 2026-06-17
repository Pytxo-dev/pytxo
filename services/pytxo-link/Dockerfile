# Build from repository root (Railway: set Dockerfile path to services/pytxo-link/Dockerfile)
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p pytxo-link

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pytxo-link /usr/local/bin/pytxo-link
COPY services/pytxo-link/migrations /migrations
# Railway sets PORT; LINK_BIND used for local/docker-compose
ENV LINK_BIND=0.0.0.0:8787
CMD ["pytxo-link"]
