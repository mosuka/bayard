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
$ curl -X POST 'http://localhost:8000/v1/search?from=0&limit=10&facet_field=category&facet_prefix[]=/language&facet_prefix[]=/category/search' --data-binary 'description:rust' | jq .
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
      "score": 1.5945008
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
      "score": 1.5945008
    }
  ],
  "facet": {
    "category": {
      "/category/search/server": 1,
      "/language/rust": 2,
      "/category/search/library": 1
    }
  }
}
```