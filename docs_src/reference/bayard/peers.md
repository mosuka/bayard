# bayard peers

The `bayard peers` CLI shows the peer addresses of the cluster that the specified server is joining.

## USAGE

    bayard peers [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## EXAMPLES

To show peers of the cluster with default options:

```text
$ ./bin/bayard peers
```

To show peers of the cluster with options:

```text
$ ./bin/bayard peers --servers=127.0.0.1:5001
```
