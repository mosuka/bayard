# Running on Docker

Docker container image is available. Check the available version at the following URL:
- https://hub.docker.com/r/bayardsearch/bayard/tags/

## Pulling Docker container

You can pull the Docker container image with the following command:

```shell script
docker pull bayardsearch/bayard:latest
```

## Running Docker container

You can run the Docker container image with the following command:

```shell script
docker run --rm --name bayard \
    -p 5000:5000 \
    bayardsearch/bayard:latest serve
```
