# bayard leave

The `bayard leave` CLI removes the server with the specified ID from the cluster that the specified server is joining.

## USAGE

    bayard leave [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]
    -i, --id <ID>                 Node ID to be removed from the cluster that specified server is joining. [default: 1]

## EXAMPLES

To remove a server with default options:

```text
$ ./bin/bayard leave
```

To probe a server with options:

```text
$ ./bin/bayard leave --servers=127.0.0.1:5001 --id=3
```
