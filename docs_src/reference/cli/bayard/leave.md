# bayard leave

## DESCRIPTION
Delete node from the cluster

## USAGE
bayard leave [OPTIONS] [ID]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Raft service address. [default: 127.0.0.1:7000]

## ARGS
- `<ID>`  
&nbsp;&nbsp;&nbsp;&nbsp; Node ID to be removed from the cluster.

## EXAMPLES

To probe a server with options:

```shell script
$ ./bin/bayard leave --servers=127.0.0.1:5001 3
```
