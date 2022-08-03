FROM rust:1.62.1-slim-bullseye AS builder

ARG BAYARD_VERSION

WORKDIR /repo

RUN set -ex \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential \
       cmake \
       pkg-config \
       libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN rustup component add rustfmt --toolchain 1.62.1-x86_64-unknown-linux-gnu

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /

RUN set -ex \
    && apt-get update \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /repo/target/release/bayard /usr/local/bin

EXPOSE 5000 7000 9000

ENTRYPOINT [ "bayard" ]
