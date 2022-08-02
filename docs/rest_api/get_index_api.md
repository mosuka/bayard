# Get Index API

## Request

```
GET /indices/<NAME>
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to create.


## Response

```json
{
    "schema": <SCHEMA>,
    "analyzers": <ANALYZERS>,
    "index_settings": <INDEX_SETTINGS>,
    "writer_threads": <WRITER_THREADS>,
    "writer_mem_size": <WRITER_MEM_SIZE>,
    "replicas": <REPLICAS>,
    "shards": <SHARDS>
}
```

- `<SCHEMA>`: (Required, object) Schema. See [Schema](../schema.md) section for the items that can be configured.
- `<ANALYZERS>`: (Required, object) Analyzer settings. See [Analyzers](../analyzers.md) section for the items that can be configured.
- `<INDEX_SETTINGS>`: (Optional, object) Index config. See Index settings section for the items that can be configured.
- `<WRITER_THREADS>` (Required, integer) Defines the number of indexing workers that should work at the same time.
- `<WRITER_MEM_SIZE>` (Required, integer) Sets the amount of memory allocated for all indexing thread. Each thread will receive a budget of `<WRITER_MEM_SIZE> / <WRITER_NUM_THREADS>`.
- `<REPLICAS>`: (Optional, integer) Number of replicas.
- `<SHARDS>`: (Optional, String array) Name of shards.

### Index settings

Contains settings which are applied on the whole index, like presort documents.

```json
{
    "sort_by_field": <SORT_BY_FIELD>,
    "docstore_compression": <DOCSTORE_COMPRESSION>,
    "docstore_blocksize": <DOCSTORE_BLOCKSIZE>
}
```

- `<SORT_BY_FIELD>`: (Optional, Object) Field to sort by. See Sort by field section for the items that can be configured.
- `<DOCSTORE_COMPRESSION>`: (Optional, string) Compression type. The following values can be defined:
    - `none`: No compression.
    - `lz4`: Use the lz4 compressor (block format).
    - `brotli`: Use the brotli compressor.
    - `snappy`: Use the snappy compressor.
- `<DOCSTORE_BLOCKSIZE>`: (Optional, Object) The size of each block that will be compressed and written to disk.

#### Sort by field

Settings to presort the documents in an index.  
Presorting documents can greatly performance in some scenarios, by applying top n optimizations.

```json
{
    "field": <FIELD>,
    "order": <ORDER>
}
```

- `<FIELD>`: (Required, string) Field name to sort by.
- `<ORDER>`: (Required, string) Order to sort by. The following values can be defined:
    - `Asc`: Ascending order.
    - `Desc`: Descending order.


## Examples

```
% curl -XGET http://localhost:8000/indices/example | jq .
```

```json
{
  "schema": [
    {
      "name": "url",
      "type": "text",
      "options": {
        "indexing": {
          "record": "freq",
          "fieldnorms": false,
          "tokenizer": "raw"
        },
        "stored": true,
        "fast": false
      }
    },
    {
      "name": "name",
      "type": "text",
      "options": {
        "indexing": {
          "record": "position",
          "fieldnorms": false,
          "tokenizer": "default"
        },
        "stored": true,
        "fast": false
      }
    },
    {
      "name": "description",
      "type": "text",
      "options": {
        "indexing": {
          "record": "position",
          "fieldnorms": false,
          "tokenizer": "default"
        },
        "stored": true,
        "fast": false
      }
    },
    {
      "name": "popularity",
      "type": "u64",
      "options": {
        "indexed": true,
        "fieldnorms": true,
        "fast": "single",
        "stored": true
      }
    },
    {
      "name": "category",
      "type": "facet",
      "options": {
        "stored": true
      }
    },
    {
      "name": "publish_date",
      "type": "date",
      "options": {
        "indexed": true,
        "fieldnorms": true,
        "fast": "single",
        "stored": true
      }
    }
  ],
  "index_settings": {
    "docstore_compression": "none",
    "docstore_blocksize": 16384
  },
  "analyzers": {
    "default": {
      "filters": [
        {
          "args": {
            "length_limit": 40
          },
          "name": "remove_long"
        },
        {
          "name": "ascii_folding"
        },
        {
          "name": "lower_case"
        }
      ],
      "tokenizer": {
        "name": "simple"
      }
    },
    "raw": {
      "tokenizer": {
        "name": "raw"
      }
    },
    "whitespace": {
      "tokenizer": {
        "name": "whitespace"
      }
    }
  },
  "writer_threads": 1,
  "writer_mem_size": 500000000,
  "replicas": 1,
  "shards": {
    "shard_list": [
      {
        "id": "b1gNghKG",
        "version": 1658906090
      }
    ]
  }
}
```