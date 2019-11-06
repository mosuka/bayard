FROM rust:1.38.0-slim-stretch as builder

WORKDIR /repo
ADD . /repo/
RUN apt -q -q -y update && \
    apt -q -q -y install build-essential cmake golang-go && \
    make build

FROM debian:stretch-slim as runner

RUN mkdir -p /data
WORKDIR /
COPY --from=builder /repo/bin /usr/local/bin
COPY --from=builder /repo/etc/* /etc/
EXPOSE 5000
ENTRYPOINT [ "bayard" ]
CMD [ "serve" ]
