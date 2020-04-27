# Get document API

Get jdocument API gets a document with the specified ID.

## Request

```text
GET /v1/documents/<ID>
```

## Path parameters

- `<ID>`  
&nbsp;&nbsp;&nbsp;&nbsp; A unique value that identifies the document in the index.

## Examples

To get a document:

```shell script
$ curl -X GET 'http://localhost:8000/v1/documents/1' | jq .
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
