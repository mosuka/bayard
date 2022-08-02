# Search API

## Request

```
POST /indices/<NAME>search
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to search.

### Request body

```json
{
    "query": <QUERY>,
    "collection_kind": <COLLECTION_KIND>,
    "sort": <SORT>,
    "fields": <FIELDS>,
    "offset": <OFFSET>,
    "hits": <HITS>,
}
```

- `<QUERY>`: (Required, object) Query DSL. See [Query DSL](../query_dsl.md) section for the items that can be configured.
- `<COLLECTION_KIND>`: (Required, String) Collection kind. The following values can be defined:
    - `count_and_top_docs`: Collect hit count and top docs.
    - `top_docs`: Collect top docs only.
    - `count`: Collect hit count only.
- `<SORT>`: (Optional, object) Analyzer settings. See Sort section for the items that can be configured. If omitted, the default is sorting by descending score.
- `<FIELDS>`: (Required, String array) Fields to return.
- `<OFFSET>`: (Required, integer) Starting document offset.
- `<HITS>`: (Required, integer) Number of documents to retrieve.

#### Sort

Specify the sort order.

```json
{
    "field": <FIELD>,
    "order": <ORDER>
}
```

- `<FIELD>`: (Required, string) Field name to sort by.
- `<ORDER>`: (Required, string) Order to sort by. The following values can be defined:
    - `asc`: Ascending order.
    - `desc`: Descending order.

## Response

```json
{
  "total_hits": <TOTAL_HITS>,
  "documents": [
      <DOCUMENT>,
      <DOCUMENT>,
      ...
      <DOCUMENT>
}
```

- `<TOTAL_HITS>`: (integer) Total number of hits.
- `<DOCUMENT>`: (object) Retrieved document. See Document section for the items that can be retrieved.

### Document

The retrieved document is in the following JSON format

```json
{
    "id": <ID>,
    "score": <SCORE>,
    "timestamp": <TIMESTAMP>,
    "sort_value": <SORT_VALUE>,
    "fields": <FIELDS>
}
```

- `<ID>`: (string) Document ID.
- `<SCORE>`: (float) Score. When sorted by field value, this value is 0.
- `<TIMESTAMP>`: (integer) The timestamp at which the document was updated.
- `<SORT_VALUE>`: (double) Sort value. When sorted by score, this value is 0.
- `<FIELDS>`: (object) Document fields. Key/value pairs expressed in JSON that make up the document.

## Examples

```
% curl -XPOST \
       --header 'Content-Type: application/json' \
       --data-binary @./examples/query_string_query.json \
       http://localhost:8000/indices/example/search | jq .
```

```json
{
  "total_hits": 1,
  "documents": [
    {
      "id": "6",
      "score": 0,
      "timestamp": 1653748212,
      "sort_value": 1331,
      "fields": {
        "name": [
          "Rust (programming language)"
        ],
        "popularity": [
          1331
        ]
      }
    }
  ]
}
```
