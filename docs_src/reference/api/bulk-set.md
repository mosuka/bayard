# Bulk set documents API

Bulk set documents API sets documents in bulk with the specified ID and field. If specify an existing ID, it will be overwritten with the new document.

## Request

```text
PUT /v1/documents
```

## Request body

- `<DOCUMENTS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Documents expressed in JSONL format

## Example

To put documents in bulk:

```shell script
$ curl -X PUT \
       --header 'Content-Type: application/json' \
       --data-binary @./examples/bulk_put.jsonl \
       'http://localhost:8000/v1/documents'
```
