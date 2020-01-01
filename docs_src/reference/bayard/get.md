# bayard get

The `bayard get` CLI gets a document with the specified ID.

## USAGE

    bayard get [OPTIONS] <DOC_ID>

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]
    -i, --id <ID>                 A unique value that identifies the document in the index.

## EXAMPLES

To get a document with default options:

```text
$ ./bin/bayard get --id=1
```
