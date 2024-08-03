# client
FROM node:20.16 AS client
WORKDIR /workspace

COPY . .

RUN <<EOF
  cd client
  npm install
  npm run build
EOF

# server
FROM rust:1.80.0-slim AS server
WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./

RUN <<EOF
  mkdir src
  echo "" > src/lib.rs
  echo "fn main() {}" > src/main.rs
  cargo build --release
EOF

COPY . .
COPY --from=client /workspace/client/dist /workspace/client/dist

RUN <<EOF
  touch src/lib.rs
  touch src/main.rs
  cargo build --release
EOF

# output
FROM debian:12.6-slim
WORKDIR /workspace

COPY --from=server /workspace/target/release/azusa ./

CMD ["/workspace/azusa"]
