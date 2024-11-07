FROM rust:1.82 AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian

WORKDIR /app

COPY --from=builder /app/target/release/ipam .

CMD ["./ipam"]