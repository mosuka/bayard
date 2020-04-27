# Bulk delete documents API

Bulk delete documents API deletes documents in bulk with the specified ID.

## Request

```text
DELETE /v1/documents
```

## Request body

- `<DOCUMENT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Document(s) expressed in JSONL format

## Examples

To delete documents in bulk:

```shell script
$ curl -X DELETE \
    --header 'Content-Type: application/json' \
    --data-binary @./examples/bulk_delete.jsonl \
    'http://localhost:8000/v1/documents'
```
