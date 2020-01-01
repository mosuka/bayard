# bayard put

The `bayard put` CLI puts a document with the specified ID and field. If specify an existing ID, it will be overwritten
with the new document.

## USAGE

    bayard put [OPTIONS] <DOC_ID> <FIELDS>

## FLAGS

    -b, --bulk       A flag indicating whether or not to put documents in bulk.
    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]
    -i, --id <ID>                 A unique value that identifies the document in the index. If specified, the existing
                                  document ID in the document is overwritten.
    -f, --file <FILE>             File path that document(s) expressed in JSON or JSONL format.

## EXAMPLES

To put a document:

```text
$ ./bin/bayard put --id=1 --file=./examples/doc_1.json
```

To put documents in bulk:

```text
$ ./bin/bayard put --bulk --file=./examples/bulk_put.jsonl
```
