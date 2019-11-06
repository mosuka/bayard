# Bayard

[![Join the chat at https://gitter.im/bayard-search/bayard](https://badges.gitter.im/bayard-search/bayard.svg)](https://gitter.im/bayard-search/bayard?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Bayard is a full-text search and indexing server written in [Rust](https://www.rust-lang.org/) built on top of [Tantivy](https://github.com/tantivy-search/tantivy) that implements [The Raft Consensus Algorithm](https://raft.github.io/) by [raft-rs](https://github.com/tikv/raft-rs) and [The gRPC](https://grpc.io/) ([HTTP/2](https://en.wikipedia.org/wiki/HTTP/2) + [Protocol Buffers](https://developers.google.com/protocol-buffers)) by [grpc-rs](https://github.com/tikv/grpc-rs) and [rust-protobuf](https://github.com/stepancheg/rust-protobuf).  
Achieves consensus across all the nodes, ensures every change made to the system is made to a quorum of nodes.  
Bayard makes easy for programmers to develop search applications with advanced features and high availability.


## Features

- Full-text search/indexing
- Index replication
- Bringing up a cluster
- Command line interface is available


## Building Bayard

```text
$ make build
```


## Starting in standalone mode (single node cluster)

Running node in standalone mode is easy. See following command:

```text
$ ./bin/bayard serve
```

### Indexing document

Indexing a document is as following:

```text
$ ./bin/bayard set 1 '{"text":"Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust."}'
$ ./bin/bayard set 2 '{"text":"Apache Lucene is a high-performance, full-featured text search engine library written entirely in Java."}'
$ ./bin/bayard set 3 '{"text":"Bleve is a modern text indexing library for go."}'
$ ./bin/bayard set 4 '{"text":"Whoosh is a fast, pure Python search engine library."}'
$ ./bin/bayard set 5 '{"text":"Solr is highly reliable, scalable and fault tolerant, providing distributed indexing, replication and load-balanced querying, automated failover and recovery, centralized configuration and more."}'
$ ./bin/bayard set 6 '{"text":"Elasticsearch is a distributed, open source search and analytics engine for all types of data, including textual, numerical, geospatial, structured, and unstructured."}'
$ ./bin/bayard set 7 '{"text":"Riot is Go Open Source, Distributed, Simple and efficient full text search engine."}'
$ ./bin/bayard set 8 '{"text":"Blast is a full text search and indexing server, written in Go, built on top of Bleve."}'
$ ./bin/bayard set 9 '{"text":"Toshi is meant to be a full-text search engine similar to Elasticsearch. Toshi strives to be to Elasticsearch what Tantivy is to Lucene."}'
$ ./bin/bayard set 10 '{"text":"Sonic is a fast, lightweight and schema-less search backend."}'
$ ./bin/bayard set 11 '{"text":"Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."}'
```

### Getting document

Getting a document is as following:

```text
$ ./bin/bayard get 11 | jq .
```

You can see the result in JSON format. The result of the above command is:

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

### Searching documents

Searching documents is as like following:

```
$ ./bin/bayard search text:"search engine" | jq .
```

You can see the result in JSON format. The result of the above command is:

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

### Deleting document

```
$ ./bin/bayard delete 11
```


## Starting in cluster mode (3-node cluster)

Bayard can easily bring up a cluster. Running in standalone is not fault tolerant. If you need to improve fault tolerance, start two more nodes as follows:

```
$ ./bin/bayard serve \
    --host=0.0.0.0 \
    --port=5001 \
    --id=1 \
    --peers="1=0.0.0.0:5001" \
    --data-directory=./data/1 \
    --schema-file=./etc/schema.json \
    --unique-key-field-name=id

$ ./bin/bayard serve \
    --host=0.0.0.0 \
    --port=5002 \
    --id=2 \
    --peers="1=0.0.0.0:5001,2=0.0.0.0:5002" \
    --leader-id=1 \
    --data-directory=./data/2 \
    --schema-file=./etc/schema.json \
    --unique-key-field-name=id

$ ./bin/bayard serve \
    --host=0.0.0.0 \
    --port=5003 \
    --id=3 \
    --peers="1=0.0.0.0:5001,2=0.0.0.0:5002,3=0.0.0.0:5003" \
    --leader-id=1 \
    --data-directory=./data/3 \
    --schema-file=./etc/schema.json \
    --unique-key-field-name=id
```

Above example shows each Bayard node running on the same host, so each node must listen on different ports. This would not be necessary if each node ran on a different host.  
Recommend 3 or more odd number of nodes in the cluster. In failure scenarios, data loss is inevitable, so avoid deploying single nodes.

### Remove a node from a cluster

If one of the nodes in a cluster goes down due to a hardware failure and raft logs and metadata is lost, that node cannot join the cluster again.

```
$ ./bin/bayard leave \
    --host=127.0.0.1 \
    --port=5001 \
    --id=3 \
    --peers="1=0.0.0.0:5001,2=0.0.0.0:5002" \
    --leader-id=1
```


## Bayard on Docker

### Pulling Docker container image from docker.io

You can pull the Docker container image already registered in docker.io like so:

```
$ docker pull bayardsearch/bayard:latest
```

Check the available version at the following URL:
https://hub.docker.com/r/bayardsearch/bayard/tags/

### Running Docker container

Running a Bayard on Docker like so:

```
$ docker run --rm --name bayard \
    -p 5000:5000 \
    bayardsearch/bayard:latest
```
