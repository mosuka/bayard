# bayard peers

The `bayard peers` CLI shows the peer addresses of the cluster that the specified server is joining.

## USAGE

    bayard peers [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --server <IP:PORT>    Server address in an existing cluster. [default: 127.0.0.1:5000]

## EXAMPLES

To show peers of the cluster with default options:

```text
$ ./bin/bayard peers
```

To show peers of the cluster with options:

```text
$ ./bin/bayard peers --server=127.0.0.1:5001
```
