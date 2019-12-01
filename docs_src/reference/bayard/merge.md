# bayard merge

The `bayard merge` CLI merges fragmented segments in the index.

## USAGE

    bayard merge [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## EXAMPLES

To merge segments in the index with default options:

```text
$ ./bin/bayard merge
```

To merge segments in the index with options:

```text
$ ./bin/bayard merge --servers=127.0.0.1:5001
```
