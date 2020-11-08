FROM rust:1.46.0-slim-buster AS builder

ARG BAYARD_REST_VERSION

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

RUN cargo install bayard-rest --root=./ --vers=${BAYARD_REST_VERSION}


FROM debian:buster-slim

WORKDIR /

RUN set -ex \
    && apt-get update \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /repo/bin/* /usr/local/bin/

EXPOSE 8000

ENTRYPOINT [ "bayard-rest" ]
CMD [ ]
