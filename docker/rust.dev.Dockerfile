FROM rust:1-bookworm

WORKDIR /workspace

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch --locked

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo fetch
