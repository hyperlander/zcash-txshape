# Build stage
FROM rust:1.75-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY cmd ./cmd

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zcash-txshape /usr/local/bin/
COPY config.toml /etc/zcash-txshape/config.toml
ENV ZCASH_TXSHAPE_CONFIG=/etc/zcash-txshape/config.toml
ENTRYPOINT ["/usr/local/bin/zcash-txshape"]
CMD ["--help"]
