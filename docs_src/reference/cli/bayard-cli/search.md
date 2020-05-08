# bayard-cli search

## DESCRIPTION
Search documents from index server

## USAGE
bayard-cli search [FLAGS] [OPTIONS] <QUERY>

## FLAGS
- `-c`, `--exclude-count`  
&nbsp;&nbsp;&nbsp;&nbsp; A flag indicating whether or not to exclude hit count in the search results.

- `-d`, `--exclude-docs`  
&nbsp;&nbsp;&nbsp;&nbsp; A flag indicating whether or not to exclude hit documents in the search results

- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

- `-f`, `--from` `<FROM>`  
&nbsp;&nbsp;&nbsp;&nbsp; Start position of fetching results. [default: 0]

- `-l`, `--limit` `<LIMIT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Limitation of amount that document to be returned. [default: 10]

- `-F`, `--facet-field` `<FACET_FIELD>`  
&nbsp;&nbsp;&nbsp;&nbsp; Hierarchical facet field name. [default: ]

- `-V`, `--facet-prefix` `<FACET_PREFIX>...`  
&nbsp;&nbsp;&nbsp;&nbsp; Hierarchical facet field value prefix.

## ARGS
- `<QUERY>`  
&nbsp;&nbsp;&nbsp;&nbsp; Query string to search the index.

## EXAMPLES

To search documents from the index with options:

```shell script
$ bayard-cli search \
             --server=0.0.0.0:5001 \
             --facet-field=category \
             --facet-prefix=/category/search \
             --facet-prefix=/language \
             description:rust | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "count": 2,
  "docs": [
    {
      "fields": {
        "_id": [
          "8"
        ],
        "category": [
          "/category/search/library",
          "/language/rust"
        ],
        "description": [
          "Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust."
        ],
        "name": [
          "Tantivy"
        ],
        "popularity": [
          3142
        ],
        "timestamp": [
          "2019-12-19T01:07:00+00:00"
        ],
        "url": [
          "https://github.com/tantivy-search/tantivy"
        ]
      },
      "score": 1.5722498
    },
    {
      "fields": {
        "_id": [
          "1"
        ],
        "category": [
          "/category/search/server",
          "/language/rust"
        ],
        "description": [
          "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."
        ],
        "name": [
          "Bayard"
        ],
        "popularity": [
          1152
        ],
        "timestamp": [
          "2019-12-19T01:41:00+00:00"
        ],
        "url": [
          "https://github.com/bayard-search/bayard"
        ]
      },
      "score": 1.5331805
    }
  ],
  "facet": {
    "category": {
      "/language/rust": 2,
      "/category/search/library": 1,
      "/category/search/server": 1
    }
  }
}
```
