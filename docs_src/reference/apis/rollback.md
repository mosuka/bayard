# Rollback API

Rollback API rolls back any updates made to the index to the last committed state.

## Request

```text
GET /index/rollback
```

## Examples

To rollback an index:

```text
$ curl -X GET 'http://localhost:8000/index/rollback'
```
