# syntax=docker/dockerfile:1.6

FROM rust:1-alpine3.23 AS chef

RUN cargo install cargo-chef --locked

FROM chef AS planner

WORKDIR /app

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/app/target \
  cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/app/target \
  cargo test --locked

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/app/target \
  cargo build --release --locked --bin media-server \
  && cp -f target/release/media-server /app/media-server

FROM alpine:3.23

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/media-server /usr/local/bin/media-server

ENV MEDIA_SERVER_CONFIG_PATH=/etc/media-server/config.yml

EXPOSE 8080

CMD ["/usr/local/bin/media-server"]
