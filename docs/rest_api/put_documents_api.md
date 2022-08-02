# Put Documents API

```
PUT /indices/<NAME>/documents
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to put documents.

### Request body

```json
{"id": <ID>, "fields": <FIELDS>}
{"id": <ID>, "fields": <FIELDS>}
{"id": <ID>, "fields": <FIELDS>}
...
{"id": <ID>, "fields": <FIELDS>}
```

- `<ID>`: (Required, String) Document ID to be unique in the index.
- `<FIELDS>`: (Required, object) Document fields. Key/value pairs expressed in JSON that make up the document.

## Response

```json
{
}
```

## Examples

```
% curl -XPUT \
       --header 'Content-Type: application/json' \
       --data-binary '
       {"id":"1", "fields": {"text":"This is an example document 1."}}
       {"id":"2", "fields": {"text":"This is an example document 2."}}
       {"id":"3", "fields": {"text":"This is an example document 3."}}
       ' \
       http://localhost:8000/indices/example/documents
```
