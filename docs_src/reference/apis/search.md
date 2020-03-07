# Search API

Search API searches documents from the index.

## Request

```text
GET /index/search
```

## Query parameters

- `from`  
Start position of fetching results. If not specified, use default value. [default: 0]

- `limit`  
Limitation of amount that document to be returned. If not specified, use default value. [default: 10]

- `exclude_count`  
A flag indicating whether or not to exclude hit count in the search results. If not specified, use default value. [default: false]

- `exclude_docs`  
A flag indicating whether or not to exclude hit documents in the search results. If not specified, use default value. [default: false]

- `query`  
Query string to search the index.

- `facet_field`  
Hierarchical facet field name.

- `facet_prefix`  
Hierarchical facet field value prefix.

## Example

To search documents from the index:

```text
$ curl -X GET 'http://localhost:8000/index/search?query=search&from=0&limit=10'
```

```text
$ curl -X GET 'http://localhost:8000/index/search?query=search&from=0&limit=10&exclude_count'
```

```text
$ curl -X GET 'http://localhost:8000/index/search?query=search&from=0&limit=10&exclude_docs'
```

```text
$ curl -X GET 'http://localhost:8000/index/search?query=search&from=0&limit=10&facet_field=category&facet_prefix=/language&facet_prefix=/category/search'
```
