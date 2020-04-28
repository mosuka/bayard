# Designing schema

## Schema

Schema is a collection of field entries.

## Field entry

A field entry represents a field and its configuration.

- `name`  
&nbsp;&nbsp;&nbsp;&nbsp; A field name.

- `type`  
&nbsp;&nbsp;&nbsp;&nbsp; A field type. See [Field type](#field-type) section.

- `options`  
&nbsp;&nbsp;&nbsp;&nbsp; Options describing how the field should be indexed. See [Options](#options) section.

## Field type

A field type describes the type of a field as well as how it should be handled.

- `text`  
&nbsp;&nbsp;&nbsp;&nbsp; String field type configuration. It can specify [text options](#text-options).

- `u64`  
&nbsp;&nbsp;&nbsp;&nbsp; Unsigned 64-bits integers field type configuration. It can specify [numeric options](#numeric-options).

- `i64`  
&nbsp;&nbsp;&nbsp;&nbsp; Signed 64-bits integers 64 field type configuration. It can specify [numeric options](#numeric-options).

- `f64`  
&nbsp;&nbsp;&nbsp;&nbsp; 64-bits float 64 field type configuration. It can specify [numeric options](#numeric-options).

- `date`  
&nbsp;&nbsp;&nbsp;&nbsp; Signed 64-bits Date 64 field type configuration. It can specify [numeric options](#numeric-options).

- `hierarchical_facet`  
&nbsp;&nbsp;&nbsp;&nbsp; Hierarchical Facet.

- `bytes`  
&nbsp;&nbsp;&nbsp;&nbsp; Bytes. (one per document)

## Options

### Text options

Configuration defining indexing for a text field.  
It defines the amount of information that should be stored about the presence of a term in a document.
Essentially, should be store the term frequency and/or the positions, the name of the tokenizer that should be used to process the field.

- `indexing`
  - `record`
    - `basic`  
&nbsp;&nbsp;&nbsp;&nbsp; Records only the document IDs.

    - `freq`  
&nbsp;&nbsp;&nbsp;&nbsp; Records the document ids as well as the term frequency. The term frequency can help giving better scoring of the documents.

    - `position`  
&nbsp;&nbsp;&nbsp;&nbsp; Records the document id, the term frequency and the positions of the occurences in the document. Positions are required to run phrase queries.

  - `tokenizer`  
&nbsp;&nbsp;&nbsp;&nbsp; Specify a text analyzer. See [Configure text analyzers](configure_text_analyzers.md#text-analyzers).
  
- `stored`
  - `true`  
&nbsp;&nbsp;&nbsp;&nbsp; Text is to be stored.

  - `false`  
&nbsp;&nbsp;&nbsp;&nbsp; Text is not to be stored.

### Numeric options

Configuration defining indexing for a numeric field.  

- `indexed`  
  - `true`  
&nbsp;&nbsp;&nbsp;&nbsp; Value is to be indexed.

  - `false`  
&nbsp;&nbsp;&nbsp;&nbsp; Value is not to be indexed.

- `stored`  
  - `true`  
&nbsp;&nbsp;&nbsp;&nbsp; Value is to be stored.

  - `false`  
&nbsp;&nbsp;&nbsp;&nbsp; Value is not to be stored.

- `fast`:
  - `single`  
&nbsp;&nbsp;&nbsp;&nbsp; The document must have exactly one value associated to the document.

  - `multi`  
&nbsp;&nbsp;&nbsp;&nbsp; The document can have any number of values associated to the document. This is more memory and CPU expensive than the SingleValue solution.

## Example schema

Here is a sample schema:

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
