# Analyzers

Analyzer is used for text analysis when indexing text fields.  
This section describes the analyzers and how they are defined that supported by Bayard.

## Analyzer

The analyzer consists of one tokenizer and one or more token filters. Analyzer entries are defined in JSON as follows:

```json
{
    <NAME>: {
        "tokenizer": <TOKENIZER>,
        "filters": [
            <FILTER>,
            <FILTER>,
            ...
        ]
    }
}
```

- `<NAME>`: (String, Required) Name of the analyzer to be used when indexing and searching in the text field. Must be unique within the index.
- `<TOKENIZER>`: (Object, Required) Advanced settings for the tokenizer used in the analyzer.
- `<FILTER>`: (Object, Optional) Advanced settings for the token filter used in the analyzer.


## Tokenizer

Tokenizers are responsible for breaking unstructured natural text into lexical units or tokens. A tokenizer is defined in JSON as follows:

```json
{
    "name": <NAME>,
    "args": <ARGS>
}
```

- `<NAME>`: (String, Required) Name of the tokenizer to be used in the analyzer. Available values are `raw`, `simple`, `whitespace`, `ngram`, `facet` and `lindera`.
- `<ARGS>`: (Object, Optional) Advanced settings for the tokenizer.

### Raw tokenizer

For each value of the field, emit a single unprocessed token. A raw tokenizer is defined in JSON as follows:

```json
{
    "name": "raw"
}
```

### Simple tokenizer

Tokenize the text by splitting on whitespaces and punctuation.  A simple tokenizer is defined in JSON as follows:

```json
{
    "name": "simple"
}
```

### Ngram tokenizer

Tokenize the text by splitting words into n-grams of the given size(s). With this tokenizer, the `position` is always 0. A ngram tokenizer is defined in JSON as follows:

```json
{
    "name": "ngram",
    "args": {
        "min_gram": 1,
        "max_gram": 2,
        "prefix_only": false
    }
}
```

- `min_gram`: (Integer, Required) Min size of the n-gram.
- `max_gram`: (Integer, Required) Max size of the n-gram.
- `prefix_only`: (Boolean, Optional) If true, will only parse the leading edge of the input.

### Facet tokenizer

Process a facet binary representation and emits a token for all of its parent. A facet tokenizer is defined in JSON as follows:

```json
{
    "name": "facet"
}
```

### Whitespace tokenizer

Tokenize the text by splitting on whitespaces. A facet tokenizer is defined in JSON as follows:

```json
{
    "name": "whitespace"
}
```

### Lindera tokenizer

Chinese, Japanese and Korean text tokenizer based on [Lindera](https://github.com/lindera-morphology/lindera). A lindera tokenizer is defined in JSON as follows:

Example for Chinese:

```json
{
    "name": "lindera",
    "args": {
        "dict_type": "cc-cedict",
        "mode": "normal"
    }
}
```

Example for Japanese:

```json
{
    "name": "lindera",
    "args": {
        "dict_type": "ipadic",
        "mode": {
            "decompose": {
                "kanji_penalty_length_threshold": 2,
                "kanji_penalty_length_penalty": 3000,
                "other_penalty_length_threshold": 7,
                "other_penalty_length_penalty": 1700
            }
        }
    }
}
```

Example for Korean:

```json
{
    "name": "lindera",
    "args": {
        "dict_type": "ko-dic",
        "mode": "normal"
    }
}
```

- `dict_type`: (String, Required) Dictionary type. Available values are `cc-cedict` (Chinese), `ipadic` (Japanese) and `ko-dic` (Korean).
- `mode`: (String or Object, Required) Analysis mode. Available values are `normal` and `decompose`.
    - `normal`: Normal mode. Outputs tokens of lexical units registered in the morphological dictionary.
    - `decompose`: Decompose mode. It only works with `ipadic`. Outputs tokens of lexical units registered in the morphological dictionary, and if the token is determined to be a compound word, the token is further decomposed. The following parameters are used to determine whether to decompose or not:
        - `kanji_penalty_length_threshold`: (Integer, Required) The length threshold of the Kanji characters.
        - `kanji_penalty_length_penalty`: (Integer, Required) The penalty of the Kanji character.
        - `other_penalty_length_threshold`: (Integer, Required) The length threshold of the other characters.
        - `other_penalty_length_penalty`: (Integer, Required) The penalty of the other characters.

## Token filter

Token filters examine a stream of tokens and keep them, transform them or discard them, depending on the filter type being used. A tokenizer is defined in JSON as follows:

```json
{
    "name": <NAME>,
    "args": <ARGS>
}
```

- `<NAME>`: (String, Required) Name of the token filter to be used in the analyzer. Available values are `alpha_num`, `ascii_folding`, `lower_case`, `remove_long`, `stemming` and `stop_word`.
- `<ARGS>`: (Object, Optional) Advanced settings for the token filter.

### Alphanumeric only token filter

Alphanumeric only token filter removes all tokens that contain non ascii alphanumeric characters. An alphanumeric only token filter is defined in JSON as follows:

```json
{
    "name": "alpha_num_only"
}
```

### ASCII folding token filter

ASCII folding token filter converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. An ASCII folding token filter is defined in JSON as follows:

```json
{
    "name": "ascii_folding"
}
```

### Lower case token filter

Lower case token filter converts lowercase terms. A lower case token filter is defined in JSON as follows:

```json
{
    "name": "lower_case"
}
```

### Remove long token filter

Remove long token filter removes tokens that are longer than a given number of bytes (in UTF-8 representation). It is especially useful when indexing unconstrained content. e.g. Mail containing base-64 encoded pictures etc. A remove long token filter is defined in JSON as follows:

```json
{
  "name": "remove_long",
  "args": {
    "length_limit": 40
  }
}
```

- `length_limit`: (Integer, Required) The maximum length of the token. A limit in bytes of the UTF-8 representation.

### Stemming token filter

Stemming token filter applies the snowball stemming algorithm. A stemming token filter is defined in JSON as follows:

```json
{
    "name": "stemming",
    "args": {
        "language": "English"
    }
}
```

- `language`: (String, Required) The language of the stemming algorithm. Available values are `Arabic`, `Danish`, `Dutch`, `English`, `Finnish`, `French`, `German`, `Greek`, `Hungarian`, `Italian`, `Norwegian`, `Portuguese`, `Romanian`, `Russian`, `Spanish`, `Swedish`, `Tamil` and `Turkish`.

### Stop word token filter

Stop word token filter removes stop words from the token stream. A stop word token filter is defined in JSON as follows:

```json
{
    "name": "stop_word",
    "args": {
        "words": [
            "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into",
            "is", "it", "no", "not", "of", "on", "or", "such", "that", "the", "their", "then",
            "there", "these", "they", "this", "to", "was", "will", "with"
        ]
    }
}
```

- `words`: (Array, Required) The list of stop words.

## Examples

The following is an example for English analyzer.

```json
{
    "tokenizer": {
        "name": "simple"
    },
    "filters": [
        {
            "name": "remove_long",
            "args": {
                "length_limit": 40
            }
        },
        {
            "name": "ascii_folding"
        },
        {
            "name": "lower_case"
        },
        {
            "name": "stemming",
            "args": {
                "language": "English"
            }
        },
        {
            "name": "stop_word",
            "args": {
                "words": [
                    "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if",
                    "in", "into", "is", "it", "no", "not", "of", "on", "or", "such",
                    "that", "the", "their", "then", "there", "these", "they", "this",
                    "to", "was", "will", "with"
                ]
            }
        }
    ]
}
```
