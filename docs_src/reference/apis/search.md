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

- `query`  
Query string to search the index.

## Example

To search documents from the index:

```text
$ curl -X GET 'http://localhost:8000/index/search?query=search&from=0&limit=10'
```
