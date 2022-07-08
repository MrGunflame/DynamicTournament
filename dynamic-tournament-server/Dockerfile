FROM rust:alpine as builder

RUN apk add musl-dev g++ openssl-dev
WORKDIR /app
COPY . .

RUN cargo build --bin dynamic-tournament-server --release

FROM scratch

WORKDIR /

COPY --from=builder /app/target/release/dynamic-tournament-server /bin

ENTRYPOINT ["/bin"]
