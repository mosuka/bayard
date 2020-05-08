# bayard-cli delete

## DESCRIPTION
Delete document from index server

## USAGE
bayard delete [OPTIONS] [ID]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

## ARGS
- `<ID>`  
&nbsp;&nbsp;&nbsp;&nbsp; A unique ID that identifies the document in the index server.

## EXAMPLES

To delete a document:

```shell script
$ bayard-cli delete --server=0.0.0.0:5001 1
```
