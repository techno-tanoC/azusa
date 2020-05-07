FROM rust:1.42.0-slim-buster

WORKDIR /build

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && \
    apt install -y --no-install-recommends libssl-dev pkg-config build-essential musl-tools

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY . .
RUN touch src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM node:12.16.1-alpine3.11

WORKDIR /build

COPY . .
RUN yarn install && yarn build


# FROM debian:buster-slim
FROM alpine:3.11

WORKDIR /app

COPY config config

COPY --from=0 /build/target/x86_64-unknown-linux-musl/release/azusa azusa
COPY --from=1 /build/assets assets

RUN chown -R 1000:1000 config assets azusa
USER 1000:1000
CMD ["./azusa"]
