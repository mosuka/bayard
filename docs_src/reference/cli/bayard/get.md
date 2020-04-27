# bayard get

## DESCRIPTION
Get document from index server

## USAGE
bayard get [OPTIONS] [ID]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

## ARGS
- `<ID>`  
&nbsp;&nbsp;&nbsp;&nbsp; A unique ID that identifies the document in the index server.

## EXAMPLES

To get a document with default options:

```shell script
$ ./bin/bayard get --server=192.168.11.10:5001 1
```

You'll see the result in JSON format. The result of the above command is:

```json
{
  "_id": [
    "1"
  ],
  "category": [
    "/category/search/server",
    "/language/rust"
  ],
  "description": [
    "Bayard is a full text search and indexing server, written in Rust, built on top of Tantivy."
  ],
  "name": [
    "Bayard"
  ],
  "popularity": [
    1152
  ],
  "timestamp": [
    "2019-12-19T01:41:00+00:00"
  ],
  "url": [
    "https://github.com/bayard-search/bayard"
  ]
}
```
