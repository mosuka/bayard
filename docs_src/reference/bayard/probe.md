# bayard probe

The `bayard probe` CLI probes the server.

## USAGE

    bayard probe [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --server <IP:PORT>    Server address in an existing cluster. [default: 127.0.0.1:5000]

## EXAMPLES

To probe a server with default options:

```text
$ ./bin/bayard probe
```

To probe a server with options:

```text
$ ./bin/bayard probe --server=127.0.0.1:5001
```
