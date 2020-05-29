# bayard-rest

## DESCRIPTION
Bayard REST server

## USAGE
bayard-rest [OPTIONS]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-H`, `--host` `<HOST>`  
&nbsp;&nbsp;&nbsp;&nbsp; Hostname or IP address. [default: 0.0.0.0]

- `-p`, `--port` `<PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; HTTP service port number. [default: 8000]

- `-i`, `--index-address` `<ADDRESS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 0.0.0.0:5000]

- `-c`, `--cert-file` `<PATH>`  
&nbsp;&nbsp;&nbsp;&nbsp; Path to the TLS certificate file.

- `-k`, `--key-file` `<PATH>`  
&nbsp;&nbsp;&nbsp;&nbsp; Path to the TLS key file.

## EXAMPLES

To start a server with default options:

```shell script
$ bayard --host=192.168.1.22 \
         --port=8001 \
         --index-address=192.168.1.12:5001
```
