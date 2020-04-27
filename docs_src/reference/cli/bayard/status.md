# bayard status

## DESCRIPTION
Show system status

## USAGE
bayard status [OPTIONS]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

## EXAMPLES

To show an index schema with options:

```shell script
$ ./bin/bayard status --server=0.0.0.0:5001 | jq .
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
