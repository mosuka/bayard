# bayard put

The `bayard put` CLI puts a document with the specified ID and field. If specify an existing ID, it will be overwritten
with the new document.

## USAGE

    bayard put [OPTIONS] <DOC_ID> <FIELDS>

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...    Server addresses in an existing cluster separated by ",". If not specified, use
                                  default servers. [default: 127.0.0.1:5000]

## ARGS

    <DOC_ID>    A unique value that identifies the document in the index. If specify an existing ID, the existing
                document in the index is overwritten.
    <FIELDS>    Document fields expressed in JSON format.

## EXAMPLES

To put a document with default options:

```text
$ ./bin/bayard put 1 '{
  "url": "https://github.com/bayard-search/bayard",
  "name": "Bayard",
  "description": "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy.",
  "star": 1132,
  "facet": ["/category/search/server", "/language/rust"]
}'
```

To put a document with options:

```text
$ ./bin/bayard put --servers=127.0.0.1:5001 1 '{
  "url": "https://github.com/bayard-search/bayard",
  "name": "Bayard",
  "description": "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy.",
  "star": 1132,
  "facet": ["/category/search/server", "/language/rust"]
}'
```
