# Getting started

## Starting in standalone mode (Single node cluster)

Running node in standalone mode is easy. You can start server with the following command:

```text
$ ./bin/bayard start 1
```

## Getting schema

You can confirm current schema with the following command:

```text
$ ./bin/bayard schema | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
[
  {
    "name": "_id",
    "type": "text",
    "options": {
      "indexing": {
        "record": "basic",
        "tokenizer": "raw"
      },
      "stored": true
    }
  },
  {
    "name": "url",
    "type": "text",
    "options": {
      "indexing": {
        "record": "freq",
        "tokenizer": "default"
      },
      "stored": true
    }
  },
  {
    "name": "name",
    "type": "text",
    "options": {
      "indexing": {
        "record": "position",
        "tokenizer": "en_stem"
      },
      "stored": true
    }
  },
  {
    "name": "description",
    "type": "text",
    "options": {
      "indexing": {
        "record": "position",
        "tokenizer": "en_stem"
      },
      "stored": true
    }
  },
  {
    "name": "popularity",
    "type": "u64",
    "options": {
      "indexed": true,
      "fast": "single",
      "stored": true
    }
  },
  {
    "name": "category",
    "type": "hierarchical_facet"
  },
  {
    "name": "timestamp",
    "type": "date",
    "options": {
      "indexed": true,
      "fast": "single",
      "stored": true
    }
  }
]
```

## Indexing document

You can index document with the following command:

```text
$ cat ./examples/doc_1.json | xargs -0 ./bin/bayard set 1
$ ./bin/bayard commit
```

## Getting document

You can get document with the following command:

```text
$ ./bin/bayard get 1 | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
{
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
}
```

## Indexing documents in bulk

You can index documents in bulk with the following command:

```text
$ cat ./examples/bulk_put.jsonl | xargs -0 ./bin/bayard bulk-set
$ ./bin/bayard commit
```

## Searching documents

You can search documents with the following command:

```text
$ ./bin/bayard search --facet-field=category --facet-prefix=/category/search --facet-prefix=/language description:rust | jq .
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

## Deleting document

You can delete document with the following command:

```text
$ ./bin/bayard delete 1
$ ./bin/bayard commit
```

## Deleting documents in bulk

You can delete documents in bulk with the following command:

```text
$ cat ./examples/bulk_delete.jsonl | xargs -0 ./bin/bayard bulk-delete
$ ./bin/bayard commit
```
