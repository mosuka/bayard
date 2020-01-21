ARG RUST_VERSION=1.39.0


FROM rust:${RUST_VERSION}-slim-stretch AS builder

WORKDIR /repo

RUN set -ex \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        cmake \
        # For protobuf
        golang-go \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies.
# COPY ./Cargo.toml ./Cargo.lock ./
COPY ./Cargo.toml ./
RUN mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && touch src/lib.rs \
    && cargo build --release

COPY . ./
RUN make build


FROM debian:stretch-slim

RUN mkdir -p /data
WORKDIR /
COPY --from=builder /repo/bin /usr/local/bin
COPY --from=builder /repo/etc/* /etc/
EXPOSE 5000
ENTRYPOINT [ "bayard" ]
CMD [ "serve" ]
