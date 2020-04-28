# Accessing over the HTTP

Bayard supports gRPC connections, but some users may want to use the traditional RESTful API over HTTP. Bayard REST server is useful in such cases.


## Using Gateway

Starting a REST server is easy.

```text
$ ./bin/bayard-rest start --port=8000 --server=0.0.0.0:5001
```

## REST API

See following documents:
- [API](reference/api.md)
