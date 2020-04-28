# Running on Docker

See the available Docker container image version at the following URL:
- [https://hub.docker.com/r/bayardsearch/bayard/tags/](https://hub.docker.com/r/bayardsearch/bayard/tags/)

## Pulling Docker container

You can pull the Docker container image with the following command:

```text
$ docker pull bayardsearch/bayard:latest
```

## Running Docker container

You can run the Docker container image with the following command:

```text
$ docker run --rm --name bayard \
    -p 5000:5000 -p 7000:7000\
    bayardsearch/bayard:latest start
```
