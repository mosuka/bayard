# Create Index API

## Request

```
PUT /indices/<NAME>
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to create.

### Request body

```json
{
    "schema": <SCHEMA>,
    "analyzers": <ANALYZERS>,
    "index_settings": <INDEX_SETTINGS>,
    "writer_threads": <WRITER_THREADS>,
    "writer_mem_size": <WRITER_MEM_SIZE>,
    "num_replicas": <NUM_REPLICAS>,
    "num_shards": <NUM_SHARDS>,
    "shards": <SHARDS>,
}
```

- `<SCHEMA>`: (Required, object) Schema. See [Schema](../schema.md) section for the items that can be configured.
- `<ANALYZERS>`: (Required, object) Analyzer settings. See [Analyzers](../analyzers.md) section for the items that can be configured.
- `<INDEX_SETTINGS>`: (Optional, object) Index config. See Index settings section for the items that can be configured.
- `<WRITER_THREADS>` (Optional, integer) Defines the number of indexing workers that should work at the same time.
- `<WRITER_MEM_SIZE>` (Optional, integer) Sets the amount of memory allocated for all indexing thread. Each thread will receive a budget of `<WRITER_MEM_SIZE> / <WRITER_NUM_THREADS>`.
- `<NUM_REPLICAS>`: (Optional, integer) Number of replicas.
- `<NUM_SHARDS>`: (Optional, integer) Number of shards.
- `<SHARDS>`: (Optional, String array) Shard list. If omitted, it will be generated automatically according to the number of shards.

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
    - `none`: No compression. (default)
    - `lz4`: Use the lz4 compressor (block format).
    - `brotli`: Use the brotli compressor.
    - `snappy`: Use the snappy compressor.
- `<DOCSTORE_BLOCKSIZE>`: (Optional, Object) The size of each block that will be compressed and written to disk. default is 16384.

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

## Response

```json
{
}
```

## Examples

```
% curl -XPUT \
       --header 'Content-Type: application/json' \
       --data-binary @./examples/meta.json \
       http://localhost:8000/indices/example
```
