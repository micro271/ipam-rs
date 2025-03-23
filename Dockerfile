FROM rust:1.85-slim AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:stable-slim

RUN useradd -m ipam

USER ipam

WORKDIR /app

COPY --from=builder --chown=ipam:ipam /app/target/release/ipam .

CMD ["./ipam"]