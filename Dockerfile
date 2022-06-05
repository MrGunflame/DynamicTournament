FROM rust:alpine as builder

RUN apk add musl-dev g++
WORKDIR /app
COPY . .

RUN cargo build --bin dynamic-tournament-server --release

FROM scratch

WORKDIR /

COPY --from=builder /app/target/release/dynamic-tournament-server /bin
COPY dynamic-tournament-server/users.json /

ENTRYPOINT ["/bin"]
