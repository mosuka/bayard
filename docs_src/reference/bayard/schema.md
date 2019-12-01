# bayard schema

The `bayard schema` CLI shows the index schema that the server applied.

## USAGE

    bayard schema [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## EXAMPLES

To show the index schema with default options:

```text
$ ./bin/bayard schema
```

To show the index schema with options:

```text
$ ./bin/bayard schema --servers=127.0.0.1:5001
```
