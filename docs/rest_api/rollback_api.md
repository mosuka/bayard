# Rollback API

## Request

```
GET /indices/<NAME>/rollback
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to rollback changes.

## Response

```json
{
}
```

## Examples

```
% curl -XGET http://localhost:8000/indices/example/rollback
```
