# bayard schedule

The `bayard schedule` CLI starts the job scheduler.

## USAGE

    bayard schedule [OPTIONS]

## FLAGS

    -h, --help       Prints help information.
    -v, --version    Prints version information.

## OPTIONS

    -s, --servers <IP:PORT>...        Server addresses in an existing cluster separated by ",". If not specified, use
                                      default servers. [default: 127.0.0.1:5000]
    -c, --commit <COMMIT_SCHEDULE>    Schedule for automatic commit in a cron-like format. If not specified, use default
                                      schedule. [default: 0/10 * * * * * *]
    -m, --merge <MERGE_SCHEDULE>      Schedule for automatic merge in a cron-like format. If not specified, use default
                                      schedule. [default: 0 0 2 * * * *]

## SCHEDULE FORMAT

The scheduling format is as follows:

```text
sec   min   hour   day-of-month   month   day-of-week   year
*     *     *      *              *       *             *
```

## EXAMPLES

To start job scheduler with default options:

```text
$ ./bin/bayard schedule
```

To start job scheduler with options:

```text
$ ./bin/bayard schedule --commit="0/10 * * * * * *" --merge="0 0 2 * * * *"
```
