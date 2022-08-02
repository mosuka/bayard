# Bring up a multi-node cluster

## Starting in cluster mode (3-node cluster)

Phalanx is a master node-less distributed search engine without external software is required. You can easily bring up a cluster by adding a command flags.

```
% bayard --bind-address=0.0.0.0:2001 \
         --grpc-address=0.0.0.0:5001 \
         --http-address=0.0.0.0:8001 \
         --data-directory=/tmp/bayard1
```

```
% bayard --bind-address=0.0.0.0:2002 \
         --grpc-address=0.0.0.0:5002 \
         --http-address=0.0.0.0:8002 \
         --data-directory=/tmp/bayard2 \
         --seed-address=0.0.0.0:2001
```

```
% bayard --bind-address=0.0.0.0:2003 \
         --grpc-address=0.0.0.0:5003 \
         --http-address=0.0.0.0:8003 \
         --data-directory=/tmp/bayard3 \
         --seed-address=0.0.0.0:2001
```

The above commands run servers on the same host, so each server must listen on a different port. This would not be necessary if each server runs on a different host.  
When deploying to a single host, if that host goes down due to hardware failure, all of the servers in the cluster will be stopped, so recommend deploying to a different host.

## Cluster nodes

You can check the peers in the cluster with the following command:

```
$ curl -XGET http://localhost:8001/cluster/nodes | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "nodes": [
    {
      "address": "0.0.0.0:2001",
      "metadata": {
        "grpc_address": "0.0.0.0:5001",
        "http_address": "0.0.0.0:8001"
      }
    },
    {
      "address": "0.0.0.0:2002",
      "metadata": {
        "grpc_address": "0.0.0.0:5002",
        "http_address": "0.0.0.0:8002"
      }
    },
    {
      "address": "0.0.0.0:2003",
      "metadata": {
        "grpc_address": "0.0.0.0:5003",
        "http_address": "0.0.0.0:8003"
      }
    }
  ]
}
```
