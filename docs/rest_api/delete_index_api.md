# Delete Index API

## Request

```
DELETE /indices/<NAME>
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to delete.

## Response

```json
{
}
```

## Examples

```
% curl -XDELETE http://localhost:8000/indices/example
```
