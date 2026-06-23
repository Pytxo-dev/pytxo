# Pytxo Ultra managed-inference proxy (ADR-0019)
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY services/pytxo-proxy ./services/pytxo-proxy
RUN cargo build --release -p pytxo-proxy

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pytxo-proxy /usr/local/bin/pytxo-proxy
ENV PORT=8790
EXPOSE 8790
CMD ["pytxo-proxy"]
