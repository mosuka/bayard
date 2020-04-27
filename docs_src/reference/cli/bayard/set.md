# bayard set

## DESCRIPTION
Set document to index server

## USAGE
bayard bulk-set [OPTIONS] [DOCS]

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

To put a document:

```shell script
$ cat ./examples/doc_1.json | xargs -0 ./bin/bayard set 1
```
