# bayard rollback

The `bayard rollback` CLI rolls back any updates made to the index to the last committed state.

## USAGE

    bayard rollback [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## EXAMPLES

To rollback an index with default options:

```text
$ ./bin/bayard rollback
```

To rollback an index with options:

```text
$ ./bin/bayard rollback --servers=127.0.0.1:5001
```
