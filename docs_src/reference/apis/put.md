# Put API

Put API puts a document with the specified ID and field. If specify an existing ID, it will be overwritten with the new document.

## Request

```text
PUT /index/docs/<DOC_ID>
PUT /index/docs
```

## Path parameters

- `<DOC_ID>`  
A unique value that identifies the document in the index. If specify an existing ID, the existing document in the index is overwritten.

## Request body

- `<DOCUMENT>`  
Document(s) expressed in JSON or JSONL format

## Example

To put a document:

```text
$ curl -X PUT \
    --header 'Content-Type: application/json' \
    --data-binary @./examples/doc_1.json \
    'http://localhost:8000/index/docs/1'
```

To put documents in bulk:

```text
$ curl -X PUT \
    --header 'Content-Type: application/json' \
    --data-binary @./examples/bulk_put.jsonl \
    'http://localhost:8000/index/docs'
```
