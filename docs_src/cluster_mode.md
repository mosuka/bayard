# Cluster mode

Bayard supports booting in cluster mode by itself. No external software is required, and you can easily bring up a cluster by adding a command flags.


## Starting in cluster mode (3-node cluster)

Running in standalone is not fault tolerant. If you need to improve fault tolerance, start servers in cluster mode.
You can start servers in cluster mode with the following command:

```text
$ ./bin/bayard serve \
    --id=1 \
    --host=0.0.0.0 \
    --port=5001 \
    --data-directory=./data/1 \
    --schema-file=./etc/schema.json
```

```text
$ ./bin/bayard serve \
    --id=2 \
    --host=0.0.0.0 \
    --port=5002 \
    --peers="1=0.0.0.0:5001" \
    --data-directory=./data/2 \
    --schema-file=./etc/schema.json
```

```text
$ ./bin/bayard serve \
    --id=3 \
    --host=0.0.0.0 \
    --port=5003 \
    --peers="1=0.0.0.0:5001,2=0.0.0.0:5002" \
    --data-directory=./data/3 \
    --schema-file=./etc/schema.json
```

The above commands run servers on the same host, so each server must listen on a different port. This would not be necessary if each server runs on a different host.
Recommend 3 or more odd number of servers in the cluster to avoid split-brain.  
When deploying to a single host, if that host goes down due to hardware failure, all of the servers in the cluster will be stopped, so recommend deploying to a different host.

## Cluster peers

You can check the peers in the cluster with the following command:

```text
$ ./bin/bayard peers --servers localhost:5001 | jq .
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "1": "0.0.0.0:5001",
  "2": "0.0.0.0:5002",
  "3": "0.0.0.0:5003"
}
```

## Remove a server from a cluster

If one of the servers in a cluster goes down due to a hardware failure and raft logs and metadata is lost, that server cannot join the cluster again.  
If you want the server to join the cluster again, you must remove it from the cluster.  
The following command deletes the server with `id=3` from the cluster:

```text
$ ./bin/bayard leave \
    --servers=127.0.0.1:5001 \
    --id=3
```
