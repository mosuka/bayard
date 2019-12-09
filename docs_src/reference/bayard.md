# bayard

The `bayard` CLI manages server, cluster and index.

## USAGE

    bayard <SUBCOMMAND>

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## SUBCOMMANDS

    serve       The `bayard serve` CLI starts the server.
    probe       The `bayard probe` CLI probes the server.
    peers       The `bayard peers` CLI shows the peer addresses of the cluster that the specified server is joining.
    metrics     The `bayard metrics` CLI shows the server metrics of the specified server. The metrics are output in
                Prometheus exposition format.
    leave       The `bayard leave` CLI removes the server with the specified ID from the cluster that the specified
                server is joining.
    put         The `bayard put` CLI puts a document with the specified ID and field. If specify an existing ID, it
                will be overwritten with the new document.
    get         The `bayard get` CLI gets a document with the specified ID.
    delete      The `bayard delete` CLI deletes a document with the specified ID.
    commit      The `bayard commit` CLI commits updates made to the index.
    rollback    The `bayard rollback` CLI rolls back any updates made to the index to the last committed state.
    merge       The `bayard merge` CLI merges fragmented segments in the index.
    search      The `bayard search` CLI searches documents from the index.
    schema      The `bayard schema` CLI shows the index schema that the server applied.
    schedule    The `bayard schedule` CLI starts the job scheduler.
    gateway     The `bayard gateway` CLI starts a gateway for access the server over HTTP.
    help        Prints this message or the help of the given subcommand(s)

## EXAMPLES

To print version information:

```text
$ ./bin/bayard -v
```
