# Getting started

## Starting in standalone mode (Single node cluster)

Running node in standalone mode is easy. You can start server with the following command:

```text
$ ./bin/bayard serve
```

You'll see a startup message like following:

```text
[2019-11-27T00:30:45Z INFO bayard::server::server src/server/server.rs:119] listening on 0.0.0.0:5000
[2019-11-27T00:30:45Z INFO raft::raft /Users/m-osuka/.cargo/registry/src/github.com-1ecc6299db9ec823/raft-0.4.3/src/raft.rs:723]  became follower at term 0
[2019-11-27T00:30:45Z INFO raft::raft /Users/m-osuka/.cargo/registry/src/github.com-1ecc6299db9ec823/raft-0.4.3/src/raft.rs:295]  newRaft [peers: [1], term: 0, commit: 0, applied: 0, last_index: 0, last_term: 0]
[2019-11-27T00:30:45Z INFO raft::raft /Users/m-osuka/.cargo/registry/src/github.com-1ecc6299db9ec823/raft-0.4.3/src/raft.rs:723]  became follower at term 1

[...]

[2019-11-27T00:30:48Z INFO raft::raft /Users/m-osuka/.cargo/registry/src/github.com-1ecc6299db9ec823/raft-0.4.3/src/raft.rs:1094]  is starting a new election at term 1
[2019-11-27T00:30:48Z INFO raft::raft /Users/m-osuka/.cargo/registry/src/github.com-1ecc6299db9ec823/raft-0.4.3/src/raft.rs:743]  became candidate at term 2
[2019-11-27T00:30:48Z INFO raft::raft /Users/m-osuka/.cargo/registry/src/github.com-1ecc6299db9ec823/raft-0.4.3/src/raft.rs:858]  received MsgRequestVoteResponse from 1 at term 2
[2019-11-27T00:30:48Z INFO raft::raft /Users/m-osuka/.cargo/registry/src/github.com-1ecc6299db9ec823/raft-0.4.3/src/raft.rs:793]  became leader at term 2
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
    "name": "id",
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
    "name": "text",
    "type": "text",
    "options": {
      "indexing": {
        "record": "position",
        "tokenizer": "en_stem"
      },
      "stored": true
    }
  }
]
```

## Indexing document

You can index documents with the following command:

```text
$ ./bin/bayard put 1 '{"url":"https://github.com/bayard-search/bayard","name":"Bayard","description":"Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy.","star":1132,"category":"/search/server"}'
$ ./bin/bayard put 2 '{"url":"https://lucene.apache.org/solr/","name":"Apache Solr","description":"Solr is highly reliable, scalable and fault tolerant, providing distributed indexing, replication and load-balanced querying, automated failover and recovery, centralized configuration and more.","star":3112,"category":"/search/server"}'
$ ./bin/bayard put 3 '{"url":"https://www.elastic.co/products/elasticsearch","name":"Elasticsearch","description":"Elasticsearch is a distributed, open source search and analytics engine for all types of data, including textual, numerical, geospatial, structured, and unstructured.","star":45871,"category":"/search/server"}'
$ ./bin/bayard put 4 '{"url":"https://github.com/mosuka/blast","name":"Blast","description":"Blast is a full text search and indexing server, written in Go, built on top of Bleve.","star":644,"category":"/search/server"}'
$ ./bin/bayard put 5 '{"url":"https://github.com/go-ego/riot","name":"Riot","description":"Riot is Go Open Source, Distributed, Simple and efficient full text search engine.","star":4936,"category":"/search/server"}'
$ ./bin/bayard put 6 '{"url":"https://github.com/toshi-search/Toshi","name":"Toshi","description":"Toshi is meant to be a full-text search engine similar to Elasticsearch. Toshi strives to be to Elasticsearch what Tantivy is to Lucene.","star":2433,"category":"/search/server"}'
$ ./bin/bayard put 7 '{"url":"https://github.com/valeriansaliou/sonic","name":"Sonic","description":"Sonic is a fast, lightweight and schema-less search backend.","star":7842,"category":"/search/server"}'
$ ./bin/bayard put 8 '{"url":"https://github.com/tantivy-search/tantivy","name":"Tantivy","description":"Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust.","star":3100,"category":"/search/library"}'
$ ./bin/bayard put 9 '{"url":"https://lucene.apache.org/","name":"Lucene","description":"Apache Lucene is a high-performance, full-featured text search engine library written entirely in Java.","star":3112,"category":"/search/library"}'
$ ./bin/bayard put 10 '{"url":"https://blevesearch.com/","name":"Bleve","description":"Bleve is a modern text indexing library for go.","star":6185,"category":"/search/library"}'
$ ./bin/bayard put 11 '{"url":"https://bitbucket.org/mchaput/whoosh/wiki/Home","name":"Whoosh","description":"Whoosh is a fast, pure Python search engine library.","star":0,"category":"/search/library"}'
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
  "id": [
    "1"
  ],
  "text": [
    "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."
  ]
}
```

## Searching documents

You can search documents with the following command:

```text
$ ./bin/bayard search description:"rust" | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "count": 2,
  "docs": [
    {
      "fields": {
        "id": [
          "8"
        ],
        "text": [
          "Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust."
        ]
      },
      "score": 1.5722498
    },
    {
      "fields": {
        "id": [
          "1"
        ],
        "text": [
          "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."
        ]
      },
      "score": 1.5331805
    }
  ]
}
```

## Deleting document

You can delete document with the following command:

```text
$ ./bin/bayard delete 1
$ ./bin/bayard delete 2
$ ./bin/bayard delete 3
$ ./bin/bayard delete 4
$ ./bin/bayard delete 5
$ ./bin/bayard delete 6
$ ./bin/bayard delete 7
$ ./bin/bayard delete 8
$ ./bin/bayard delete 9
$ ./bin/bayard delete 10
$ ./bin/bayard delete 11
$ ./bin/bayard commit
```
