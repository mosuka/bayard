# bayard metrics

## DESCRIPTION
Shows system metrics

## USAGE
bayard metrics [OPTIONS]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s,` `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

## EXAMPLES

To show metrics with default options:

```shell script
$ ./bin/bayard metrics
```

To show metrics with options:

```shell script
$ ./bin/bayard metrics --server=127.0.0.1:5001
```

You'll see the result in Prometheus exposition format. The result of the above command is:

```text
# HELP bayard_applies_total Total number of applies.
# TYPE bayard_applies_total counter
bayard_applies_total{func="bulk_set"} 1
bayard_applies_total{func="commit"} 1
# HELP bayard_apply_duration_seconds The apply latencies in seconds.
# TYPE bayard_apply_duration_seconds histogram
bayard_apply_duration_seconds_bucket{func="bulk_set",le="0.005"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="0.01"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="0.025"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="0.05"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="0.1"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="0.25"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="0.5"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="1"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="2.5"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="5"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="10"} 1
bayard_apply_duration_seconds_bucket{func="bulk_set",le="+Inf"} 1
bayard_apply_duration_seconds_sum{func="bulk_set"} 0.001098082
bayard_apply_duration_seconds_count{func="bulk_set"} 1
bayard_apply_duration_seconds_bucket{func="commit",le="0.005"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="0.01"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="0.025"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="0.05"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="0.1"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="0.25"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="0.5"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="1"} 0
bayard_apply_duration_seconds_bucket{func="commit",le="2.5"} 1
bayard_apply_duration_seconds_bucket{func="commit",le="5"} 1
bayard_apply_duration_seconds_bucket{func="commit",le="10"} 1
bayard_apply_duration_seconds_bucket{func="commit",le="+Inf"} 1
bayard_apply_duration_seconds_sum{func="commit"} 1.727736793
bayard_apply_duration_seconds_count{func="commit"} 1
```
