# Status API

Status API shows the cluster that the specified server is joining.

## Request

```text
GET /v1/status
```

## Examples

To show peers of the cluster:

```shell script
$ curl -X GET 'http://localhost:8000/v1/status'
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "leader": 1,
  "nodes": [
    {
      "address": {
        "index_address": "0.0.0.0:5001",
        "raft_address": "0.0.0.0:7001"
      },
      "id": 1
    },
    {
      "address": {
        "index_address": "0.0.0.0:5002",
        "raft_address": "0.0.0.0:7002"
      },
      "id": 2
    },
    {
      "address": {
        "index_address": "0.0.0.0:5003",
        "raft_address": "0.0.0.0:7003"
      },
      "id": 3
    }
  ],
  "status": "OK"
}
```
