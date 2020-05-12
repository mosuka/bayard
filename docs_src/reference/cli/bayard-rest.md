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
&nbsp;&nbsp;&nbsp;&nbsp; Node address. [default: 0.0.0.0]

- `-p`, `--port` `<PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; HTTP service port number. [default: 8000]

- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

- `-w`, `--http-worker-threads` `<HTTP_WORKER_THREADS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Number of HTTP worker threads. By default http server uses number of available logical cpu as threads count. [default: 8]

## EXAMPLES

To start a server with default options:

```shell script
$ bayard --host=192.168.1.22 \
         --port=8001 \
         --server=192.168.1.12:5001
```
