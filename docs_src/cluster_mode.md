# Cluster mode

Bayard supports booting in cluster mode by itself. No external software is required, and you can easily bring up a cluster by adding a command flags.


## Starting in cluster mode (3-node cluster)

Running in standalone is not fault tolerant. If you need to improve fault tolerance, start servers in cluster mode.
You can start servers in cluster mode with the following command:

```shell script
./bin/bayard serve \
    --host=0.0.0.0 \
    --port=5001 \
    --id=1 \
    --peers="1=0.0.0.0:5001" \
    --data-directory=./data/1 \
    --schema-file=./etc/schema.json \
    --unique-key-field-name=id
```

```shell script
./bin/bayard serve \
    --host=0.0.0.0 \
    --port=5002 \
    --id=2 \
    --peers="1=0.0.0.0:5001,2=0.0.0.0:5002" \
    --leader-id=1 \
    --data-directory=./data/2 \
    --schema-file=./etc/schema.json \
    --unique-key-field-name=id
```

```shell script
./bin/bayard serve \
    --host=0.0.0.0 \
    --port=5003 \
    --id=3 \
    --peers="1=0.0.0.0:5001,2=0.0.0.0:5002,3=0.0.0.0:5003" \
    --leader-id=1 \
    --data-directory=./data/3 \
    --schema-file=./etc/schema.json \
    --unique-key-field-name=id
```

The above commands run servers on the same host, so each server must listen on a different port. This would not be necessary if each server runs on a different host.
Recommend 3 or more odd number of servers in the cluster to avoid split-brain.  
When deploying to a single host, if that host goes down due to hardware failure, all of the servers in the cluster will be stopped, so recommend deploying to a different host.
