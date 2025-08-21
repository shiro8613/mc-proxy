FROM --platform=$TARGETOS/$TARGETARCH rust:1.89.0 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release

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