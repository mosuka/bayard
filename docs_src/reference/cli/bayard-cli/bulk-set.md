# bayard-cli bulk-set

## DESCRIPTION
Set documents to index server in bulk

## USAGE
bayard-cli bulk-set [OPTIONS] [DOCS]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

## ARGS
- `<DOCS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Document containing the unique ID to be indexed.

## EXAMPLES

To put documents in bulk:

```shell script
$ cat ./examples/bulk_put.jsonl | xargs -0 bayard-cli bulk-set
```
