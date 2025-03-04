FROM rust:1.85-slim AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:stable-slim

WORKDIR /app

COPY --from=builder /app/target/release/ipam .

CMD ["./ipam"]