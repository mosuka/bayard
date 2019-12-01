# bayard probe

The `bayard probe` CLI probes the server.

## USAGE

    bayard probe [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## EXAMPLES

To probe a server with default options:

```text
$ ./bin/bayard probe
```

To probe a server with options:

```text
$ ./bin/bayard probe --servers=127.0.0.1:5001
```
