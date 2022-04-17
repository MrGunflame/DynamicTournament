FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo build --bin dynamic-tournament-server --release

FROM debian:stable-slim

WORKDIR /app

COPY --from=builder /app/target/release/dynamic-tournament-server /app/bin
COPY dynamic-tournament-server/config.toml /app
COPY dynamic-tournament-server/users.json /app

ENTRYPOINT ["/app/bin"]
