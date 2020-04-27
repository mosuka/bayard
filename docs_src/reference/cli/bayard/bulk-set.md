# bayard bulk-set

## DESCRIPTION
Set documents to index server in bulk

## USAGE
bayard set [OPTIONS] [ARGS]

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

- `<FIELDS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Fields of document to be indexed.

## EXAMPLES

To put documents in bulk:

```shell script
$ cat ./examples/bulk_put.jsonl | xargs -0 ./bin/bayard bulk-set
```
