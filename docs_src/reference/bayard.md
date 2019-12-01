# bayard

The `bayard` CLI manages server, cluster and index.

## USAGE

    bayard <SUBCOMMAND>

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## SUBCOMMANDS

    serve       Start server
    probe       Probe a server
    peers       Get cluster peers
    metrics     Get metrics
    leave       Remove a node from a cluster
    set         Index document
    get         Get document
    delete      Delete document
    commit      Commit index
    rollback    Rollback index
    merge       Merge index
    search      Search documents
    schema      Get schema
    schedule    Schedule jobs
    gateway     Schedule jobs
    help        Prints this message or the help of the given subcommand(s)

## EXAMPLES

To print version information:

```text
$ ./bin/bayard -V
```
