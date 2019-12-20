# bayard metrics

The `bayard metrics` CLI shows the server metrics of the specified server. The metrics are output in Prometheus exposition format.

## USAGE

    bayard metrics [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

   -s, --server <IP:PORT>    Server address in an existing cluster. [default: 127.0.0.1:5000]

## EXAMPLES

To show metrics with default options:

```text
$ ./bin/bayard metrics
```

To show metrics with options:

```text
$ ./bin/bayard metrics --server=127.0.0.1:5001
```
