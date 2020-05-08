# bayard-cli set

## DESCRIPTION
Set document to index server

## USAGE
bayard-cli set [OPTIONS] [ARGS]

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

To put a document:

```shell script
$ cat ./examples/doc_1.json | xargs -0 bayard-cli set 1
```
