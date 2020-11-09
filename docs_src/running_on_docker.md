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
    -p 5000:5000 -p 7000:7000 -p 9000:9000 \
    bayardsearch/bayard:latest \
    --data-directory=/tmp/bayard \
    --schema-file /etc/bayard/schema.json \
    --tokenizer-file /etc/bayard/tokenizer.json \
    1
```

## Running with Docker composer

Using the example `docker-composer.yml`, you can start the index node and the REST server with the following command:

```text
$ docker-compose up
```
