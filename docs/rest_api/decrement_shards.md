# Dencrememt Shards API

## Request

```
PUT /indices/<NAME>/shards
```

### Path parameters
- `<NAME>`: (Required, string) Name of the index you want to create.

### Request body

```json
{
    "amount": <AMOUNT>
}
```

- `<AMOUNT>`: (Required, integer) Number of shards to be decreased.


## Response

```json
{
}
```

## Examples

```
% curl -XDELETE \
       --header 'Content-Type: application/json' \
       --data-binary '
       {
           "amount": 1
       }
       ' \
       http://localhost:8000/indices/example/shards
```
