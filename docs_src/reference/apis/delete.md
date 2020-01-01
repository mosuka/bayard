# Delete API

Delete API deletes a document with the specified ID.

## Request

```text
DELETE /index/docs/<DOC_ID>
DELETE /index/docs
```

## Path parameters

- `<DOC_ID>`  
A unique value that identifies the document in the index.

## Request body

- `<DOCUMENT>`  
Document(s) expressed in JSONL format

## Examples

To delete a document:

```text
$ curl -X DELETE 'http://localhost:8000/index/docs/1'
```

To delete documents in bulk:

```text
$ curl -X DELETE \
    --header 'Content-Type: application/json' \
    --data-binary @./examples/bulk_delete.jsonl \
    'http://localhost:8000/index/docs'
```