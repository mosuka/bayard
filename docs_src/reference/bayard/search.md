# bayard search

The `bayard search` CLI searches documents from the index.

## USAGE

    bayard search [OPTIONS] <QUERY>

## FLAGS

    -c, --exclude-count    A flag indicating whether or not to exclude hit count in the search results.
    -d, --exclude-docs     A flag indicating whether or not to exclude hit documents in the search results
    -h, --help             Prints help information.
    -v, --version          Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...              Server addresses in an existing cluster separated by ",". If not specified,
                                            use default servers. [default: 127.0.0.1:5000]
    -f, --from <FROM>                       Start position of fetching results. If not specified, use default value.
                                            [default: 0]
    -l, --limit <LIMIT>                     Limitation of amount that document to be returned. If not specified, use
                                            default value. [default: 10]
    -F, --facet-field <FACET_FIELD>         Hierarchical facet field name. [default: ]
    -V, --facet-prefix <FACET_PREFIX>...    Hierarchical facet field value prefix.

## ARGS
    <QUERY>    Query string to search the index.


## EXAMPLES

To search documents from the index with default options:

```text
$ ./bin/bayard search text:"rust"
```

To search documents from the index with options:

```text
$ ./bin/bayard search --servers=127.0.0.1:5001 --from=10 --limit=20 text:"rust"
```
