# bayard gateway

The `bayard gateway` CLI starts a gateway for access the server over HTTP.

## USAGE

    bayard gateway [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -H, --host <HOST>             Host address. Must specify the host name or IP address. If not specified, use the
                                  default address. [default: 0.0.0.0]
    -P, --port <PORT>             Port number. This port is used for communication via HTTP. If not specified, use the
                                  default port. [default: 8000]
    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## EXAMPLES

To start gateway with default options:

```text
$ ./bin/bayard gateway
```

To start gateway with options:

```text
$ ./bin/bayard gateway --host=localhost --port=8080 --servers=127.0.0.1:5001
```
