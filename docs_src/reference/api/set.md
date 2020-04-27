# Set document API

Set document API sets a document with the specified ID and field. If specify an existing ID, it will be overwritten with the new document.

## Request

```text
PUT /v1/documents/<ID>
```

## Path parameters

- `<ID>`  
&nbsp;&nbsp;&nbsp;&nbsp; A unique value that identifies the document in the index. If specify an existing ID, the existing document in the index is overwritten.

## Request body

- `<DOCUMENT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Document expressed in JSON format

## Example

To put a document:

```text
$ curl -X PUT \
       --header 'Content-Type: application/json' \
       --data-binary @./examples/doc_1.json \
       'http://localhost:8000/v1/documents/1'
```
