FROM --platform=$TARGETOS/$TARGETARCH rust:1.89.0 AS builder

WORKDIR /app

COPY . .

RUN cargo build --release --locked

FROM --platform=$TARGETOS/$TARGETARCH debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mc-proxy ./mc-proxy

CMD ["./mc-proxy"]
