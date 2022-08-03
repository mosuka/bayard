# Delete Documents API

```
DELETE /indices/<NAME>/documents
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to delete documents.

### Request body

```json
{"id": <ID>}
{"id": <ID>}
{"id": <ID>}
...
{"id": <ID>}
```

- `<ID>`: (Required, String) Document ID to be unique in the index.

## Response

```json
{
}
```

## Examples

```
% curl -XDELETE \
       --header 'Content-Type: application/json' \
       --data-binary '
       {"id":"1"}
       {"id":"2"}
       {"id":"3"}
       ' \
       http://localhost:8000/indices/example/documents
```
