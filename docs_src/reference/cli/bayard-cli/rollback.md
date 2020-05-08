# bayard-cli rollback

## DESCRIPTION
Rollback index

## USAGE
bayard-cli rollback [OPTIONS]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

## EXAMPLES

To rollback an index with options:

```shell script
$ bayard-cli rollback --server=127.0.0.1:5001
```
