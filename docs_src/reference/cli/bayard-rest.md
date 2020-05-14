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

- `-i`, `--index-server` `<ADDRESS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 0.0.0.0:5000]

- `-w`, `--worker-threads` `<THREADS>`  
&nbsp;&nbsp;&nbsp;&nbsp; Number of HTTP worker threads. By default http server uses number of available logical cpu as threads count. [default: 8]

- `-o`, `--cors-origin` `<ORIGIN>`  
&nbsp;&nbsp;&nbsp;&nbsp; Path to the TLS certificate file

- `-m`, `--cors-methods` `<METHODS>...`  
&nbsp;&nbsp;&nbsp;&nbsp; Set a list of methods which the allowed origins are allowed to access for requests.

- `-l`, `--cors-headers` `<METHODS>...`  
&nbsp;&nbsp;&nbsp;&nbsp; Set a list of header field names which can be used when this resource is accessed by allowed origins.

- `-c`, `--cert-file` `<PATH>`  
&nbsp;&nbsp;&nbsp;&nbsp; Path to the TLS certificate file.

- `-k`, `--key-file` `<PATH>`  
&nbsp;&nbsp;&nbsp;&nbsp; Path to the TLS key file.

## EXAMPLES

To start a server with default options:

```shell script
$ bayard --host=192.168.1.22 \
         --port=8001 \
         --index-server=192.168.1.12:5001
```
