FROM rust:slim AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

# Build the actual binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/celia-media /usr/local/bin/celia-media

ENV CELIA_CONFIG_PATH=/etc/celia-media/config.yml

EXPOSE 8080

CMD ["celia-media"]
