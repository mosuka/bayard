# Query DSL

Bayard provides a full Query DSL (Domain Specific Language) based on JSON to define queries. 
This section describes the queries and how they are defined that supported by Bayard.

## Defining a query

A query by the Query DSL is basically defined by the following JSON.

```json
{
    "kind": <KIND>,
    "options": <OPTIONS>
}
```
- `<KIND>`: (String, Required) The kind of query. Available values are `all`, `boolean`, `boost`, `fuzzy_term`, `phrase`, `query_string`, `range`, `regex` and `term`.
- `<OPTIONS>`: (Object, Optional) Advanced settings for the query.

### All query

Query that matches all of the documents. All of the document get the score 1.0. An all query is defined in JSON as follows:

```json
{
    "kind": "all"
}
```

### Boolean query

The boolean query returns a set of documents that matches the Boolean combination of constituent subqueries. A boolean query is defined in JSON as follows:

```json
{
    "kind": "boost",
    "options": {
        "subqueries": [
            {
                "occurrence": "must",
                "query": {
                    "kind": "term",
                    "options": {
                        "term": "hello",
                        "field": "description"
                    }
                }
            },
            {
                "occurrence": "must_not",
                "query": {
                    "kind": "term",
                    "options": {
                        "term": "bye",
                        "field": "description"
                    }
                }
            },
            {
                "occurrence": "should",
                "query": {
                    "kind": "term",
                    "options": {
                        "term": "hi",
                        "field": "description"
                    }
                }
            }
        ]
    }
}
```

- `subqueries`: (Array, Required) An array of subqueries.
    - `occurrence`: (String, Required) The occurrence of the subquery. Available values are `must`, `must_not` and `should`.
    - `query`: (Object, Required) The subquery.
        - `kind`: (String, Required) The kind of the subquery. Available values are `all`, `boolean`, `boost`, `fuzzy_term`, `phrase`, `query_string`, `range`, `regex` and `term`.
        - `options`: (Object, Required) Advanced settings for the subquery.

### Boost query

Boost query is a wrapper over a query used to boost its score. The document set matched by the boost query is strictly the same as the underlying query. The score of each document, is the score of the underlying query multiplied by the `boost` factor. A boost query is defined in JSON as follows:

```json
{
    "kind": "boost",
    "options": {
        "query": {
            "kind": "term",
            "options": {
                "term": "rust",
                "field": "description"
            }
        },
        "boost": 2.0
    }
}
```

- `query`: (Object, Required) The query.
    - `kind`: (String, Required) The kind of the subquery. Available values are `all`, `boolean`, `boost`, `fuzzy_term`, `phrase`, `query_string`, `range`, `regex` and `term`.
    - `options`: (Object, Required) Advanced settings for the query.
- `boost`: (Float, Required) The boost factor.

### Fuzzy term query

A fuzzy term query matches all of the documents containing a specific term that is within Levenshtein distance. A fuzzy term query is defined in JSON as follows:

```json
{
    "kind": "fuzzy_term",
    "options": {
        "field": "description",   
        "term": "rust",
        "distance": 2,
        "transposition_cost_one": true,
        "prefix": true
    }
}
```

- `field`: (String, Required) The field to search for.
- `term`: (String, Required) The term to search for.
- `distance`: (Integer, Required) The maximum Levenshtein distance.
- `transposition_cost_one`: (Boolean, Optional) If true, the cost of transposition is 1.0. If false, the cost of transposition is 2.0.
- `prefix`: (Boolean, Optional) If true, the term is a prefix. If false, the term is a full word.

### Phrase query

Phrase query matches a specific sequence of words. For instance the phrase query for `"part time"` will match the sentence `Alan just got a part time job.` .
On the other hand it will not match the sentence. `This is my favorite part of the job.`.
Using a `PhraseQuery` on a field requires positions to be indexed for this field. A phrase query is defined in JSON as follows:

```json
{
    "kind": "phrase",
    "options": {
        "phrase_terms": [
            "multi",
            "paradigm"
        ],
        "field": "description",
        "slop": 0
    }
}
```

- `phrase_terms`: (Array, Required) An array of terms to search for. There must be at least two terms, and all terms must belong to the same field.
- `field`: (String, Required) The field to search for.
- `slop`: (Integer, Optional) The maximum number of other terms that can appear between the terms.

### Query string query

A query string query parses a given string using a query parser and searches for using queries interpreted. A query string query is defined in JSON as follows:

```json
{
    "kind": "query_string",
    "options": {
        "query": "rust",
        "default_search_fields": [
            "name",
            "description"
        ]
    }
}
```

- `query`: (String, Required) The query string.
- `default_search_fields`: (Array, Optional) An array of default search fields.

### Range query

Range query matches all documents that have at least one term within a defined range. Matched document will all get a constant `Score` of one. A range query is defined in JSON as follows:

```json
{
    "kind": "range",
    "options": {
        "field": "popularity",
        "start": 10000,
        "end": 20000
    }
}
```

- `field`: (String, Required) The field to search for.
- `start`: (Integer, Required) The start of the range.
- `end`: (Integer, Required) The end of the range.

### Regex query

Regex Query matches all of the documents containing a specific term that matches a regex pattern.
Wildcard queries (e.g. ho*se) can be achieved by converting them to their regex counterparts. A regex query is defined in JSON as follows:

```json
{
    "kind": "regex",
    "options": {
        "regex": "ru.+t",
        "field": "description"
    }
}
```

- `regex`: (String, Required) The regex pattern.
- `field`: (String, Required) The field to search for.

### Term query

Term query matches all of the documents containing a specific term. A range query is defined in JSON as follows:

```json
{
    "kind": "term",
    "options": {
        "term": "rust",
        "field": "description"
    }
}
```

- `term`: (String, Required) The term to search for.
- `field`: (String, Required) The field to search for.
