# bayard-cli schema

## DESCRIPTION
Show index schema

## USAGE
bayard-cli schema [OPTIONS]

## FLAGS
- `-h`, `--help`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints help information.

- `-v`, `--version`  
&nbsp;&nbsp;&nbsp;&nbsp; Prints version information.

## OPTIONS
- `-s`, `--server` `<IP:PORT>`  
&nbsp;&nbsp;&nbsp;&nbsp; Index service address. [default: 127.0.0.1:5000]

## EXAMPLES

To show an index schema with options:

```shell script
$ bayard-cli schema --server=127.0.0.1:5001
```

You'll see the result in JSON format. The result of the above command is:

```json
[
  {
    "name": "_id",
    "type": "text",
    "options": {
      "indexing": {
        "record": "basic",
        "tokenizer": "raw"
      },
      "stored": true
    }
  },
  {
    "name": "url",
    "type": "text",
    "options": {
      "indexing": {
        "record": "freq",
        "tokenizer": "default"
      },
      "stored": true
    }
  },
  {
    "name": "name",
    "type": "text",
    "options": {
      "indexing": {
        "record": "position",
        "tokenizer": "en_stem"
      },
      "stored": true
    }
  },
  {
    "name": "description",
    "type": "text",
    "options": {
      "indexing": {
        "record": "position",
        "tokenizer": "en_stem"
      },
      "stored": true
    }
  },
  {
    "name": "popularity",
    "type": "u64",
    "options": {
      "indexed": true,
      "fast": "single",
      "stored": true
    }
  },
  {
    "name": "category",
    "type": "hierarchical_facet"
  },
  {
    "name": "timestamp",
    "type": "date",
    "options": {
      "indexed": true,
      "fast": "single",
      "stored": true
    }
  }
]
```
