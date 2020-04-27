# Accessing over the HTTP

Bayard supports gRPC connections, but some users may want to use the traditional RESTful API over HTTP. Gateways are useful in such cases.


## Using Gateway

Starting a Gateway is easy.

```text
$ ./bin/bayard gateway
```

If you want to start a Gateway that connects to Bayard running in cluster mode, we recommend that you specify a flag as follows:

```text
$ ./bin/bayard gateway --servers=127.0.0.1:5001,127.0.0.1:5002,127.0.0.1:5003
```


## REST API

See following documents:
- [APIs](reference/api.md)
