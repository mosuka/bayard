# Search API

Search API searches documents from the index.

## Request

```text
GET /v1/search
```

## Query parameters

- `from`  
Start position of fetching results. If not specified, use default value. [default: 0]

- `limit`  
Limitation of amount that document to be returned. If not specified, use default value. [default: 10]

- `exclude_count`  
A flag indicating whether or not to exclude hit count in the search results. If not specified, use default value. [default: false]

- `exclude_docs`  
A flag indicating whether or not to exclude hit documents in the search results. If not specified, use default value. [default: false]

- `query`  
Query string to search the index.

- `facet_field`  
Hierarchical facet field name.

- `facet_prefix`  
Hierarchical facet field value prefix.

## Example

To search documents from the index:

```text
$ curl -X POST 'http://localhost:8000/v1/search?query=search&from=0&limit=10&facet_field=category&facet_prefix=/language&facet_prefix=/category/search'
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "count": 9,
  "docs": [
    {
      "fields": {
        "_id": [
          "3"
        ],
        "category": [
          "/category/search/server",
          "/language/java"
        ],
        "description": [
          "Elasticsearch is a distributed, open source search and analytics engine for all types of data, including textual, numerical, geospatial, structured, and unstructured."
        ],
        "name": [
          "Elasticsearch"
        ],
        "popularity": [
          46054
        ],
        "timestamp": [
          "2019-12-18T23:19:00+00:00"
        ],
        "url": [
          "https://www.elastic.co/products/elasticsearch"
        ]
      },
      "score": 10.516742
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
      "score": 1.4125781
    },
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
      "score": 1.4125781
    },
    {
      "fields": {
        "_id": [
          "6"
        ],
        "category": [
          "/category/search/server",
          "/language/rust"
        ],
        "description": [
          "Toshi is meant to be a full-text search engine similar to Elasticsearch. Toshi strives to be to Elasticsearch what Tantivy is to Lucene."
        ],
        "name": [
          "Toshi"
        ],
        "popularity": [
          2448
        ],
        "timestamp": [
          "2019-12-01T19:00:00+00:00"
        ],
        "url": [
          "https://github.com/toshi-search/Toshi"
        ]
      },
      "score": 1.389255
    },
    {
      "fields": {
        "_id": [
          "11"
        ],
        "category": [
          "/category/search/library",
          "/language/python"
        ],
        "description": [
          "Whoosh is a fast, pure Python search engine library."
        ],
        "name": [
          "Whoosh"
        ],
        "popularity": [
          0
        ],
        "timestamp": [
          "2019-10-07T20:30:26+00:00"
        ],
        "url": [
          "https://bitbucket.org/mchaput/whoosh/wiki/Home"
        ]
      },
      "score": 0.2778122
    },
    {
      "fields": {
        "_id": [
          "7"
        ],
        "category": [
          "/category/search/server",
          "/language/rust"
        ],
        "description": [
          "Sonic is a fast, lightweight and schema-less search backend."
        ],
        "name": [
          "Sonic"
        ],
        "popularity": [
          7895
        ],
        "timestamp": [
          "2019-12-10T14:13:00+00:00"
        ],
        "url": [
          "https://github.com/valeriansaliou/sonic"
        ]
      },
      "score": 0.2778122
    },
    {
      "fields": {
        "_id": [
          "4"
        ],
        "category": [
          "/category/search/server",
          "/language/go"
        ],
        "description": [
          "Blast is a full text search and indexing server, written in Go, built on top of Bleve."
        ],
        "name": [
          "Blast"
        ],
        "popularity": [
          654
        ],
        "timestamp": [
          "2019-10-18T10:50:00+00:00"
        ],
        "url": [
          "https://github.com/mosuka/blast"
        ]
      },
      "score": 0.23746987
    },
    {
      "fields": {
        "_id": [
          "5"
        ],
        "category": [
          "/category/search/server",
          "/language/go"
        ],
        "description": [
          "Riot is Go Open Source, Distributed, Simple and efficient full text search engine."
        ],
        "name": [
          "Riot"
        ],
        "popularity": [
          4948
        ],
        "timestamp": [
          "2019-12-15T22:12:00+00:00"
        ],
        "url": [
          "https://github.com/go-ego/riot"
        ]
      },
      "score": 0.23746987
    },
    {
      "fields": {
        "_id": [
          "9"
        ],
        "category": [
          "/category/search/library",
          "/language/java"
        ],
        "description": [
          "Apache Lucene is a high-performance, full-featured text search engine library written entirely in Java."
        ],
        "name": [
          "Lucene"
        ],
        "popularity": [
          3135
        ],
        "timestamp": [
          "2019-12-19T05:08:00+00:00"
        ],
        "url": [
          "https://lucene.apache.org/"
        ]
      },
      "score": 0.22139496
    }
  ],
  "facet": {
    "category": {
      "/language/java": 2,
      "/language/python": 1,
      "/category/search/library": 3,
      "/language/rust": 4,
      "/category/search/server": 6,
      "/language/go": 2
    }
  }
}
```