# bayard delete

The `bayard delete` CLI deletes a document with the specified ID.

## USAGE

    bayard delete [OPTIONS] <DOC_ID>

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## ARGS

    <DOC_ID>    A unique value that identifies the document in the index.

## EXAMPLES

To delete a document with default options:

```text
$ ./bin/bayard delete 1
```

To delete a document with options:

```text
$ ./bin/bayard delete --servers=127.0.0.1:5001 1
```
