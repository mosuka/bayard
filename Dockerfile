ARG RUST_VERSION=1.42.0


FROM rust:${RUST_VERSION}-slim-stretch AS builder

WORKDIR /repo

RUN set -ex \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential \
       cmake \
       jq \
       pkg-config \
       libssl-dev \
       golang-go \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-show

COPY . ./
RUN make build


FROM debian:stretch-slim

WORKDIR /

RUN set -ex \
    && apt-get update \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /data

COPY --from=builder /repo/bin /usr/local/bin
COPY --from=builder /repo/etc/* /etc/

EXPOSE 5000 7000

ENTRYPOINT [ "bayard" ]
CMD [ "start" ]
