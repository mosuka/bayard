# Delete document API

Delete document API deletes a document with the specified ID.

## Request

```text
DELETE /v1/documents/<ID>
```

## Path parameters

- `<ID>`  
&nbsp;&nbsp;&nbsp;&nbsp; A unique value that identifies the document in the index.

## Examples

To delete a document:

```shell script
$ curl -X DELETE 'http://localhost:8000/v1/documents/1'
```
