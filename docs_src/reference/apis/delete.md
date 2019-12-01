# Delete API

Delete API deletes a document with the specified ID.

## Request

```text
DELETE /index/docs/<DOC_ID>
```

## Path parameters

- `<DOC_ID>`  
A unique value that identifies the document in the index.

## Examples

To delete a document:

```text
$ curl -X DELETE 'http://localhost:8000/index/docs/1'
```
