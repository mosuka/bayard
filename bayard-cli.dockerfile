FROM rust:1.46.0-slim-stretch AS builder

ARG BAYARD_CLI_VERSION

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
    && rm -rf /var/lib/apt/lists/*

RUN cargo install bayard-cli --root=./ --vers=${BAYARD_CLI_VERSION}


FROM debian:stretch-slim

WORKDIR /

RUN set -ex \
    && apt-get update \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /repo/bin/* /usr/local/bin/

ENTRYPOINT [ "bayard-cli" ]
CMD [ ]
