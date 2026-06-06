FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pytxo-link /usr/local/bin/pytxo-link
ENV LINK_BIND=0.0.0.0:8787
EXPOSE 8787
CMD ["pytxo-link"]
