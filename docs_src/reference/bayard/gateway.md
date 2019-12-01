# bayard gateway

The `bayard gateway` CLI starts a gateway for access the server over HTTP.

## USAGE

    bayard gateway [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

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
