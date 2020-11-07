FROM rust:1.46.0-slim-buster AS builder

ARG BAYARD_VERSION

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

COPY ./etc ./etc
RUN cargo install bayard --root=./ --vers=${BAYARD_VERSION}


FROM debian:stretch-slim

WORKDIR /

RUN set -ex \
    && apt-get update \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /repo/bin/* /usr/local/bin/
COPY --from=builder /repo/etc/* /etc/bayard/

EXPOSE 5000 7000 9000

ENTRYPOINT [ "bayard" ]
CMD [ "1" ]
