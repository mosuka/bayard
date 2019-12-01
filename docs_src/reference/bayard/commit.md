# bayard commit

The `bayard commit` CLI commits updates made to the index.

## USAGE

    bayard commit [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## EXAMPLES

To commit an index with default options:

```text
$ ./bin/bayard commit
```

To commit an index with options:

```text
$ ./bin/bayard commit --servers=127.0.0.1:5001
```
