# bayard delete

The `bayard delete` CLI deletes a document with the specified ID.

## USAGE

    bayard delete [OPTIONS] <DOC_ID>

## FLAGS

    -b, --bulk       A flag indicating whether or not to delete documents in bulk.
    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]
    -i, --id <ID>                 A unique value that identifies the document in the index.
    -f, --file <FILE>             File path that delete document(s) expressed in JSONL format.

## EXAMPLES

To delete a document:

```text
$ ./bin/bayard delete --id=1
```

To delete documents in builk:

```text
$ ./bin/bayard delete --bulk --file=./examples/bulk_delete.jsonl
```
