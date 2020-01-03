# Scheduling jobs

You can use the Job scheduler to execute periodic commits and index merges.


## Using Job scheduler

Start the job scheduler as follows:

```text
$ ./bin/bayard schedule
```

To specify job scheduler settings:

```text
$ ./bin/bayard schedule --commit="0/10 * * * * * *" --merge="0 0 2 * * * *"
```

The format of the settings is very similar to the crontab.

```text
sec   min   hour   day-of-month   month   day-of-week   year
*     *     *      *              *       *             *
```
