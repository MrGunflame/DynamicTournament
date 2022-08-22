FROM rust:latest as builder

RUN apt-get update && apt-get install -y git make
WORKDIR /app
COPY . .

RUN cd dynamic-tournament-web && make build

FROM nginx

COPY dynamic-tournament-web/docker/nginx/conf.d /etc/nginx/conf.d
COPY --from=builder /app/build/dist /app

ENTRYPOINT ["nginx"]
