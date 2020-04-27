# Rollback API

Rollback API rolls back any updates made to the index to the last committed state.

## Request

```text
GET /v1/rollback
```

## Examples

To rollback an index:

```shell script
$ curl -X GET 'http://localhost:8000/v1/rollback'
```
