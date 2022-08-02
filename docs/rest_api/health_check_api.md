# Health Check API

## Liveness probe

### Request

```
GET /healthcheck/livez
```

### Response

```json
{
  "state": "alive"
}
```

- `state`: `alive` or `dead`

### Examples

```
% curl -XGET http://localhost:8000/healthcheck/livez | jq .
```


## Readiness probe

### Request

```
GET /healthcheck/readyz
```

### Response

```json
{
  "state": "ready"
}
```

- `state`: `ready` or `not_ready`

Examples

```
% curl -XGET http://localhost:8000/healthcheck/readyz | jq .
```
