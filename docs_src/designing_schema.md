# Designing schema

## Schema

Schema is a collection of field entries.

## Field entry

A field entry represents a field and its configuration.

- name  
  A field name.

- type  
  A field type. See [Field type](#field-type) section.

- options  
  Options describing how the field should be indexed. See [Options](#options) section.

## Field type

A field type describes the type of a field as well as how it should be handled.

- "text"  
  String field type configuration. It can specify [text options](#text-options).

- "u64"  
  Unsigned 64-bits integers field type configuration. It can specify [numeric options](#numeric-options).

- "i64"  
  Signed 64-bits integers 64 field type configuration. It can specify [numeric options](#numeric-options).

- "f64"  
  64-bits float 64 field type configuration. It can specify [numeric options](#numeric-options).

- "date"  
  Signed 64-bits Date 64 field type configuration. It can specify [numeric options](#numeric-options).

- "hierarchical_facet"  
  Hierarchical Facet.

- "bytes"  
  Bytes. (one per document)

## Options

### Text options

Configuration defining indexing for a text field.  
It defines the amount of information that should be stored about the presence of a term in a document.
Essentially, should be store the term frequency and/or the positions, the name of the tokenizer that should be used to process the field.

- indexing
  - record
    - "basic"  
    Records only the document IDs.

    - "freq"  
    Records the document ids as well as the term frequency. The term frequency can help giving better scoring of the documents.

    - "position"  
    Records the document id, the term frequency and the positions of the occurences in the document. Positions are required to run phrase queries.

  - tokenizer
    - "default"  
    Chops the text on according to whitespace and punctuation, removes tokens that are too long, and lowercases tokens.
    
    - "raw"  
    Does not process nor tokenize the text.
    
    - "en_stem"  
    Like `default`, but also applies stemming on the resulting tokens. Stemming can improve the recall of your search engine.
  
- stored
  - true  
  Text is to be stored.

  - false  
  Text is not to be stored.

### Numeric options

Configuration defining indexing for a numeric field.  

- indexed  
  - true  
  Value is to be indexed.

  - false  
  Value is not to be indexed.

- stored   
  - true  
  Value is to be stored.

  - false  
  Value is not to be stored.

- fast:
  - "single"  
  The document must have exactly one value associated to the document.

  - "multi"  
  The document can have any number of values associated to the document. This is more memory and CPU expensive than the SingleValue solution.

## Example schema

Here is a sample schema:

```json
[
  {
    "name": "id",
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
    "name": "star",
    "type": "u64",
    "options": {
      "indexed": true,
      "stored": true,
      "fast": "single"
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
      "stored": true,
      "fast": "single"
    }
  }
]
```
