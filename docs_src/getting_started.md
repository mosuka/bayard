# Getting started

## Starting in standalone mode (Single node cluster)

Running node in standalone mode is easy. You can start server with the following command:

```shell script
./bin/bayard serve
```

You'll see a startup message like following:

```text
[2019-11-11T09:28:24Z INFO  tantivy::indexer::segment_updater] save metas
[2019-11-11T09:28:24Z INFO  bayard::cmd::serve] starting a server...
[2019-11-11T09:28:24Z INFO  raft::raft]  became follower at term 0
[2019-11-11T09:28:24Z INFO  raft::raft]  newRaft [peers: [1], term: 0, commit: 0, applied: 0, last_index: 0, last_term: 0]
[2019-11-11T09:28:24Z INFO  raft::raft]  became follower at term 1
[2019-11-11T09:28:24Z INFO  bayard::server::server] listening on 0.0.0.0:5000
[2019-11-11T09:28:26Z INFO  raft::raft]  is starting a new election at term 1
[2019-11-11T09:28:26Z INFO  raft::raft]  became candidate at term 2
[2019-11-11T09:28:26Z INFO  raft::raft]  received MsgRequestVoteResponse from 1 at term 2
[2019-11-11T09:28:26Z INFO  raft::raft]  became leader at term 2
```

## Indexing document

You can index documents with the following command:

```shell script
./bin/bayard set 1 '{"text":"Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust."}'
./bin/bayard set 2 '{"text":"Apache Lucene is a high-performance, full-featured text search engine library written entirely in Java."}'
./bin/bayard set 3 '{"text":"Bleve is a modern text indexing library for go."}'
./bin/bayard set 4 '{"text":"Whoosh is a fast, pure Python search engine library."}'
./bin/bayard set 5 '{"text":"Solr is highly reliable, scalable and fault tolerant, providing distributed indexing, replication and load-balanced querying, automated failover and recovery, centralized configuration and more."}'
./bin/bayard set 6 '{"text":"Elasticsearch is a distributed, open source search and analytics engine for all types of data, including textual, numerical, geospatial, structured, and unstructured."}'
./bin/bayard set 7 '{"text":"Riot is Go Open Source, Distributed, Simple and efficient full text search engine."}'
./bin/bayard set 8 '{"text":"Blast is a full text search and indexing server, written in Go, built on top of Bleve."}'
./bin/bayard set 9 '{"text":"Toshi is meant to be a full-text search engine similar to Elasticsearch. Toshi strives to be to Elasticsearch what Tantivy is to Lucene."}'
./bin/bayard set 10 '{"text":"Sonic is a fast, lightweight and schema-less search backend."}'
./bin/bayard set 11 '{"text":"Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."}'
```

## Getting document

You can get document with the following command:

```shell script
./bin/bayard get 11 | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "id": [
    "11"
  ],
  "text": [
    "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."
  ]
}
```

## Searching documents

You can search documents with the following command:

```shell script
./bin/bayard search text:"search engine" | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
[
  {
    "id": [
      "4"
    ],
    "text": [
      "Whoosh is a fast, pure Python search engine library."
    ]
  },
  {
    "id": [
      "7"
    ],
    "text": [
      "Riot is Go Open Source, Distributed, Simple and efficient full text search engine."
    ]
  },
  {
    "id": [
      "1"
    ],
    "text": [
      "Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust."
    ]
  },
  {
    "id": [
      "2"
    ],
    "text": [
      "Apache Lucene is a high-performance, full-featured text search engine library written entirely in Java."
    ]
  },
  {
    "id": [
      "6"
    ],
    "text": [
      "Elasticsearch is a distributed, open source search and analytics engine for all types of data, including textual, numerical, geospatial, structured, and unstructured."
    ]
  },
  {
    "id": [
      "9"
    ],
    "text": [
      "Toshi is meant to be a full-text search engine similar to Elasticsearch. Toshi strives to be to Elasticsearch what Tantivy is to Lucene."
    ]
  },
  {
    "id": [
      "10"
    ],
    "text": [
      "Sonic is a fast, lightweight and schema-less search backend."
    ]
  },
  {
    "id": [
      "11"
    ],
    "text": [
      "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."
    ]
  },
  {
    "id": [
      "8"
    ],
    "text": [
      "Blast is a full text search and indexing server, written in Go, built on top of Bleve."
    ]
  }
]
```

## Deleting document

You can delete document with the following command:

```shell script
./bin/bayard delete 11
```
