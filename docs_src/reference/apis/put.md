# Put API

Put API puts a document with the specified ID and field. If specify an existing ID, it will be overwritten with the new document.

## Request

```text
PUT /index/docs/<DOC_ID>
```

## Path parameters

- `<DOC_ID>`  
A unique value that identifies the document in the index. If specify an existing ID, the existing document in the index is overwritten.

## Request body

- `<FIELDS>`  
Document fields expressed in JSON format.

## Example

To put a document:

```text
$ curl -X PUT \
    --header 'Content-Type: application/json' \
    --data-binary '{"text": "Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust."}' \
    'http://localhost:8000/index/docs/1'
```
