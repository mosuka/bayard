# bayard

## DESCRIPTION
Bayard server

## USAGE
bayard [OPTIONS] [ID]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-H`, `--host` `<HOST>`  
&nbsp;&nbsp;&nbsp;&nbsp; Node address. [default: 0.0.0.0]

- `-r`, `--raft-port` `<RAFT_PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Raft service port number. [default: 7000]

- `-i`, `--index-port` `<INDEX_PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service port number [default: 5000]

- `-M`, `--metrics-port` `<METRICS_PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Metrics service port number [default: 9000]

- `-p`, `--peer-raft-address` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Raft address of a peer node running in an existing cluster.

- `-d`, `--data-directory` `<DATA_DIRECTORY>`  
&nbsp;&nbsp;&nbsp;&nbsp; Data directory. Stores index, snapshots, and raft logs. [default: ./data]

- `-s`, `--schema-file` `<SCHEMA_FILE>`  
&nbsp;&nbsp;&nbsp;&nbsp; Schema file. Must specify An existing file name. [default: ./etc/schema.json]

- `-T`, `--tokenizer-file` `<TOKENIZER_FILE>`  
&nbsp;&nbsp;&nbsp;&nbsp; Tokenizer file. Must specify An existing file name. [default: ./etc/tokenizer.json]

- `-t`, `--indexer-threads` `<INDEXER_THREADS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Number of indexer threads. By default indexer uses number of available logical cpu as threads count. [default: 8]

- `-m`, `--indexer-memory-size` `<INDEXER_MEMORY_SIZE>`  
&nbsp;&nbsp;&nbsp;&nbsp; Total memory size (in bytes) used by the indexer. [default: 1000000000]

- `-w`, `--http-worker-threads` `<HTTP_WORKER_THREADS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Number of HTTP worker threads. By default http server uses number of available logical cpu as threads count. [default: 8]

## ARGS
- `<ID>`  
&nbsp;&nbsp;&nbsp;&nbsp; Node ID.

## EXAMPLES

To start a server with default options:

```shell script
$ bayard 1
```

To start a server with options:

```shell script
$ bayard --host=0.0.0.0 \
         --raft-port=7001 \
         --index-port=5001 \
         --metrics-port=9001 \
         --data-directory=./data/node1 \
         --schema-file=./etc/schema.json \
         --tokenizer-file=./etc/tokenizer.json \
         1
```
