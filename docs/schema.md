# Schema

A schema is a definition of how an index can index documents.  
This section describes the data types supported by Bayard and how they are defined.


## Field entry

A schema consists of one or more field entries. A field entry is defined in JSON as follows:

```json
{
    "name": <NAME>,
    "type": <TYPE>,
    "options": <OPTIONS>,
}
```

- `<NAME>`: (String) Name of the field to be included in the index. Must be unique in the schema.  
- `<TYPE>`: (String) The data type of the field. The following data types can be defined:
    - `text`: Unstructured natural language text or keywords.
    - `u64`: Unsigned 64-bits integer.
    - `i64`: Signed 64-bits integers.
    - `f64`: 64-bits floating point number.
    - `date`: Date time. ISO 8601 formatted string.
    - `facet`: Hierarchical facet.
    - `bytes`: Binary data.
    - `json_object`: JSON data.
- `<OPTIONS>`: (Object) Detailed settings for each data type of field. The field options are of the following types:
    - Text option: Field options for `text` field type.
    - Numeric option: Field options for `u64`, `i64`, `f64` and `date` field types.
    - Facet option: Field options for `facet` field type.
    - Bytes option: Field options for `bytes` field type.
    - JSON object option: Field options for `json_object` field type.

### Text option

Define how a text field should be handled. A text option is defined in JSON as follows:

```json
{
    "indexing": <INDEXING>,
    "stored": <STORED>
}
```

- `<INDEXING>`: (Object) Defines how the text will be indexed.
- `<STORED>`: (Boolean) WWhether or not to store the original value. Set to true if you want to return the field values of document retrieved.

#### Indexing options

An indexing options are defined in JSON as follows:

```json
{
    "record": <RECORD>,
    "fieldnorms": <FIELD_NORMS>,
    "tokenizer": <ANALYZER>
}
```

- `<RECORD>`: (String) Describe in the schema the amount of information that should be retained during indexing. The following values can be defined:
    - `basic`: Records only the document ID.
    - `freq`: Records the document ids as well as the term frequency. The term frequency can help giving better scoring of the documents.
    - `position`: Records the document id, the term frequency and the positions of the occurrences in the document. Positions are required to run phrase queries.
- `<FIELD_NORMS>`: (Boolean) Whether or not to store the field norms.
- `<ANALYZER>`: (String) The name of the analyzer to be used for indexing. See [Analyzer](analyzers.md) section for analyzers that can be configured.


### Numeric option

Define how an `u64`, `i64`, `f64` and `date` field should be handled. A numeric option is defined in JSON as follows:

```json
{
    "indexed": <INDEXED>,
    "fieldnorms": <FIELD_NORMS>,
    "fast": <FAST>,
    "stored": <STORED>
}
```

- `<INDEXED>`: (Boolean) Whether or not to index the field.
- `<FIELD_NORMS>`: (Boolean) Whether or not to store the field norms. This attribute only has an effect if `indexed` is true.
- `<FAST>`: (String) Express whether a field is single-value or multi-valued.
    - `single`: Single-valued field. The document must have exactly one value associated to the document.
    - `multi`: Multi-valued field. The document can have any number of values associated to the document. This is more memory and CPU expensive than `single`.
- `<STORED>`: (Boolean) Whether or not to store the original value. Set to true if you want to return the field values of document retrieved.


### Facet option

Define how a facet field should be handled. A facet option is defined in JSON as follows:

```json
{
    "stored": <STORED>
}
```

- `<STORED>`: (Boolean) Whether or not to store the original value. Set to true if you want to return the field values of document retrieved.  

Note that a Facet is always indexed and stored as a fastfield.


### Bytes option

Define how an a bytes field should be handled. A bytes option is defined in JSON as follows:

```json
{
    "indexed": <INDEXED>,
    "fieldnorms": <FIELD_NORMS>,
    "fast": <FAST>,
    "stored": <STORED>
}
```

- `<INDEXED>`: (Boolean) Whether or not to index the field.
- `<FIELD_NORMS>`: (Boolean) Whether or not to store the field norms. This attribute only has an effect if `indexed` is true.
- `<FAST>`: (Boolean) Whether or not to index the field using fast field.
- `<STORED>`: (Boolean) Whether or not to store the original value. Set to true if you want to return the field values of document retrieved.


### JSON object option

Make it possible to configure how a json object field should be indexed and stored.  
A bytes option is defined in JSON object as follows:

```json
{
    "indexing": <INDEXING>,
    "stored": <STORED>
}
```

- `<INDEXING>`: (Object) Defines how the text will be indexed.
- `<STORED>`: (Boolean) WWhether or not to store the original value. Set to true if you want to return the field values of document retrieved.

#### Indexing options

An indexing options are defined in JSON as follows:

```json
{
    "record": <RECORD>,
    "fieldnorms": <FIELD_NORMS>,
    "tokenizer": <ANALYZER>
}
```

- `<RECORD>`: (String) Describe in the schema the amount of information that should be retained during indexing. The following values can be defined:
    - `basic`: Records only the document ID.
    - `freq`: Records the document ids as well as the term frequency. The term frequency can help giving better scoring of the documents.
    - `position`: Records the document id, the term frequency and the positions of the occurrences in the document. Positions are required to run phrase queries.
- `<FIELD_NORMS>`: (Boolean) Whether or not to store the field norms.
- `<ANALYZER>`: (String) The name of the analyzer to be used for indexing. See [Analyzer](analyzers.md) section for analyzers that can be configured.

## Examples

The following is an example of a text field definition.

```json
{
    "name": "description",
    "type": "text",
    "options": {
        "indexing": {
            "record": "position",
            "fieldnorms": false,
            "tokenizer": "lang_en"
        },
        "stored": true
    }
}
```

The following is an example of a numeric field definition.

```json
{
    "name": "popularity",
    "type": "u64",
    "options": {
        "indexed": true,
        "fieldnorms": false,
        "fast": "single",
        "stored": true
    }
}
```

The following is an example of a facet field definition.

```json
{
    "name": "category",
    "type": "facet",
    "options": {
        "stored": true
    }
}
```

The following is an example of a bytes field definition.

```json
{
    "name": "content",
    "type": "bytes",
    "options": {
        "indexed": true,
        "fieldnorms": false,
        "fast": true,
        "stored": true
    }
}
```

The following is an example of a json_object field definition.

```json
{
    "name": "description",
    "type": "json_object",
    "options": {
        "indexing": {
            "record": "position",
            "fieldnorms": false,
            "tokenizer": "lang_en"
        },
        "stored": true
    }
}
```

The following is a sample definition of the entire schema.

```json
[
    {
        "name": "url",
        "type": "text",
        "options": {
            "indexing": {
                "record": "freq",
                "fieldnorms": false,
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
                "fieldnorms": false,
                "tokenizer": "bigram"
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
                "fieldnorms": false,
                "tokenizer": "lang_en"
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
            "fast": "single",
            "stored": true
        }
    }
]
```

