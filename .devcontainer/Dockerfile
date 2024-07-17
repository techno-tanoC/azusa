FROM rust:1.79.0-slim
WORKDIR /workspace

RUN <<EOF
  rustup component add rustfmt
  rustup component add clippy
EOF

RUN <<EOF
  apt update
  apt install -y make git wget curl
EOF

RUN <<EOF
  curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
  cargo binstall --no-confirm cargo-watch
EOF
