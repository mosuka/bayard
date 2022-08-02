# Commit API

## Request

```
GET /indices/<NAME>/commit
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to commit changes.

## Response

```json
{
}
```

## Examples

```
% curl -XGET http://localhost:8000/indices/example/commit
```
