# Cluster Nodes API

## Request

```
GET /cluster/nodes
```

## Response

```json
{
  "nodes": [
    {
      "address": "0.0.0.0:2000",
      "metadata": {
        "grpc_address": "0.0.0.0:5000",
        "http_address": "0.0.0.0:8000"
      }
    }
  ]
}
```

- `nodes`: List of nodes in the cluster.
  - `address`: The bind address that should be bound to for internal cluster communications.
  - `metadata`: Metadata of the node.
    - `grpc_address`: The gRPC address that should be bound to for internal cluster communications and client communications.
    - `http_address`: The HTTP address that should be bound to for client communications.

## Examples

```
% curl -XGET http://localhost:8000/cluster/nodes | jq .
```
