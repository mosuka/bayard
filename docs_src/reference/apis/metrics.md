# Metrics API

Metrics API shows the server metrics of the specified server. The metrics are output in Prometheus exposition format.

## Request

```text
GET /metrics
```

## Examples

To show metrics:

```text
$ curl -X GET 'http://localhost:8000/metrics'
```
