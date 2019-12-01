# Get API

Get API gets a document with the specified ID.

## Request

```text
GET /index/docs/<DOC_ID>
```

## Path parameters

- `<DOC_ID>`  
A unique value that identifies the document in the index.

## Examples

To get a document:

```text
$ curl -X GET 'http://localhost:8000/index/docs/1'
```
