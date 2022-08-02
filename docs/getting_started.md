# Getting started

This section describes how to start Bayard on a single node and then use that node to perform basic indexing and searching.


## Starting the node

The first step is to start Bayard on a single node. This is not difficult at all.  
The following command starts Bayard with default settings.

```
% bayard
```


## Checking the node health

Let's perform a health check to see if it is running properly.  
Access the following endpoint, which will return an HTTP status code of 200 and JSON like the following if Bayard has started successfully.

[http://localhost:9921/healthcheck/livez](http://localhost:9921/healthcheck/livez)

```
% curl -XGET http://localhost:8000/healthcheck/livez | jq .
```

```json
{
  "state": "alive"
}
```


## Creating the index

A freshly launched Bayard does not have any indices. once Bayard has been launched, the next step is to create an index.  
To create an index, the following metadata must be provided.  

The format of that JSON file is shown below:
```json
{
    "schema": {...},
    "analyzers": {...},
    "index_settings": {...},
    "writer_threads": 1,
    "writer_mem_size": 500000000,
    "num_replicas": 2,
    "num_shards": 2,
}
```
However, since the JSON file for creating an index tends to mutch information, see example JSON file.
- [examples/meta.json](../examples/meta.json )

The following command combines and creates an index that named `example` those files.  

```
% curl -XPUT \
       --header 'Content-Type: application/json' \
       --data-binary @./examples/meta.json \
       http://localhost:8000/indices/example
```

Upon successful creating an index, the following file is created:

```
/tmp/Bayard
└── indices
    └── example
        ├── meta.json
        └── shards
            └── zeZcTRdA
                └── meta.json
```


## Indexing the documents

Once you have created an index, index your documents. Indexing documents is very easy.
Bayard handles indexed documents in JSON format. See the following example:

```json
{
	"id": "6",
	"fields": {
		"url": "https://en.wikipedia.org/wiki/Rust_(programming_language)",
		"name": "Rust (programming language)",
		"description": "Rust is a multi-paradigm, general-purpose programming language designed for performance and safety, especially safe concurrency. It is syntactically similar to C++, but can guarantee memory safety by using a borrow checker to validate references. It achieves memory safety without garbage collection, and reference counting is optional. It is a systems programming language with mechanisms for low-level memory management, but also offers high-level features such as functional programming.",
		"popularity": 1331,
		"category": ["/language/rust"],
		"publish_date": "2022-04-07T00:00:00+00:00"
	}
}
```

The `id` is the value that makes the document unique, and the `fields` are key/value pairs of information other than the document's `id`.  
Bayard accepts JSONL format containing multiple above JSONs.  
You can index the sample documents with the following command:

```
% curl -XPUT \
       --header 'Content-Type: application/x-ndjson' \
       --data-binary @./examples/docs.jsonl \
       http://localhost:8000/indices/example/documents
```


## Committing the index

Documents are not yet searchable after just indexing them. To actually make them searchable, you must commit the changes made to the index. 　
Commit with the following command:

```
% curl -XGET http://localhost:8000/indices/example/commit
```


## Searching the documents

Now let's search the index you have created.  
Bayard can perform a variety of queries. Although not detailed here, it supports a JSON-based query DSL.
For example, the following JSON can represent a query:

```json
{
    "name": "example",
    "shard_id": "",
    "query": {
        "kind": "query_string",
        "options": {
            "query": "rust",
            "default_search_fields": [
                "name",
                "description"
            ]
        }
    },
    "collection_kind": "count_and_top_docs",
    "sort": {
        "field": "popularity",
        "order": "desc"
    },
    "fields": [
        "id",
        "name",
        "popularity"
    ],
    "offset": 0,
    "hits": 10
}
```

You can retrieve the indexed documents with the following command:

```
% curl -XPOST \
       --header 'Content-Type: application/json' \
       --data-binary @./examples/query_string_query.json \
       http://localhost:8000/indices/example/search | jq .
```

You can retrieve the following search results:

```json
{
  "total_hits": 1,
  "documents": [
    {
      "id": "6",
      "score": 0,
      "timestamp": 1653748212,
      "sort_value": 1331,
      "fields": {
        "name": [
          "Rust (programming language)"
        ],
        "popularity": [
          1331
        ]
      }
    }
  ]
}
```
