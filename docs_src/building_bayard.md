# Building Bayard

## Requirements

The following products are required to build bayard-proto:

- Rust >= 1.39.0
- make >= 3.81
- protoc >= 3.9.2

### Install protoc-gen-rust

```shell script
$ cargo install protobuf-codegen
$ cargo install grpcio-compiler
```

### Install protoc-gen-grpc-web

```shell script
$ curl -o /usr/local/bin/protoc-gen-grpc-web -L https://github.com/grpc/grpc-web/releases/download/1.0.7/protoc-gen-grpc-web-1.0.7-darwin-x86_64
$ chmod +x /usr/local/bin/protoc-gen-grpc-web
```

## Build

Build Bayard with the following command:

```text
$ make build
```

When the build is successful, the binary file is output to the following directory:

```text
$ ls ./bin
```
