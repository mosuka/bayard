# Configure text analyzers

Bayard can analyze text by combining the prepared tokenizers and filters.

## Tokenizers

Tokenizers are responsible for breaking field data into lexical units, or tokens.

### raw

For each value of the field, emit a single unprocessed token.

```json
{
  "name": "raw"
}
```

### simple

Tokenize the text by splitting on whitespaces and punctuation.

```json
{
  "name": "simple"
}
```

### ngram

Tokenize the text by splitting words into n-grams of the given size(s).

- `min_gram`:  
&nbsp;&nbsp;&nbsp;&nbsp; Min size of the n-gram.

- `max_gram`:  
&nbsp;&nbsp;&nbsp;&nbsp; Max size of the n-gram.

- `prefix_only`:  
&nbsp;&nbsp;&nbsp;&nbsp; If true, will only parse the leading edge of the input.

```json
{
  "name": "ngram",
  "args": {
    "min_gram": 1,
    "max_gram": 3,
    "prefix_only": false
  }
}
```

### facet

Process a facet binary representation and emits a token for all of its parent.

```json
{
  "name": "facet"
}
```

### cang_jie

A Chinese tokenizer based on [jieba-rs](https://github.com/messense/jieba-rs).

- `hmm`:  
&nbsp;&nbsp;&nbsp;&nbsp; Enable HMM or not.

- `tokenizer_option`:  
&nbsp;&nbsp;&nbsp;&nbsp; Tokenizer option.

    - `all`:  
&nbsp;&nbsp;&nbsp;&nbsp; Cut the input text, return all possible words.

    - `default`:  
&nbsp;&nbsp;&nbsp;&nbsp; Cut the input text.

    - `search`:  
&nbsp;&nbsp;&nbsp;&nbsp; Cut the input text in search mode.

    - `unicode`:  
&nbsp;&nbsp;&nbsp;&nbsp; Cut the input text into UTF-8 characters.

```json
{
  "name": "cang_jie",
  "args": {
    "hmm": false,
    "tokenizer_option": "search"
  }
}
```

### lindera

A Tokenizer based on [Lindera](https://github.com/lindera-morphology/lindera).

- `mode`:  
&nbsp;&nbsp;&nbsp;&nbsp; Tokenization mode.

    - `normal`:  
&nbsp;&nbsp;&nbsp;&nbsp; Tokenize faithfully based on words registered in the dictionary. (Default)

    - `decompose`:  
&nbsp;&nbsp;&nbsp;&nbsp; Tokenize a compound noun words additionally.

- `dict`:  
&nbsp;&nbsp;&nbsp;&nbsp; Specify the pre-built dictionary directory path instead of the default dictionary (IPADIC). Please refer to the following repository for building a dictionary:  
&nbsp;&nbsp;&nbsp;&nbsp; - <a href="https://github.com/bayard-search/lindera-ipadic-builder" target="_blank">Lindera IPADIC Builder</a> (Japanese)  
&nbsp;&nbsp;&nbsp;&nbsp; - <a href="https://github.com/bayard-search/lindera-ipadic-neologd-builder" target="_blank">Lindera IPDIC NEologd Builder</a> (Japanese)  
&nbsp;&nbsp;&nbsp;&nbsp; - <a href="https://github.com/bayard-search/lindera-unidic-builder" target="_blank">Lindera UniDic Builder</a> (Japanese)  
&nbsp;&nbsp;&nbsp;&nbsp; - <a href="https://github.com/bayard-search/lindera-ko-dic-builder" target="_blank">Lindera ko-dic Builder</a> (Korean)  

```json
{
  "name": "lindera",
  "args": {
    "mode": "decompose"
  }
}
```

## Filters

Filters examine a stream of tokens and keep them, transform them or discard them, depending on the filter type being used.

### alpha_num_only

Removes all tokens that contain non ascii alphanumeric characters.

```json
{
  "name": "alpha_num_only"
}
```

### ascii_folding

Converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists.

```json
{
  "name": "ascii_folding"
}
```

### lower_case

Converts lowercase terms.

```json
{
  "name": "lower_case"
}
```

### remove_long

Removes tokens that are longer than a given number of bytes (in UTF-8 representation). It is especially useful when indexing unconstrained content. e.g. Mail containing base-64 encoded pictures etc.

- `length_limit`:  
&nbsp;&nbsp;&nbsp;&nbsp; A limit in bytes of the UTF-8 representation.

```json
{
  "name": "remove_long",
  "args": {
    "length_limit": 40
  }
}
```

### stemming

Stemming token filter. Several languages are supported. Tokens are expected to be lowercased beforehand.

- `stemmer_algorithm`:  
&nbsp;&nbsp;&nbsp;&nbsp; A given language algorithm.  

    - `arabic`
    - `danish`
    - `dutch`
    - `english`
    - `finnish`
    - `french`
    - `german`
    - `greek`
    - `hungarian`
    - `italian`
    - `norwegian`
    - `portuguese`
    - `romanian`
    - `russian`
    - `spanish`
    - `swedish`
    - `tamil`
    - `turkish`

```json
{
  "name": "stemming",
  "args": {
    "stemmer_algorithm": "english"
  }
}
```

### stop_word

Removes stop words from a token stream.

- `word`:  
&nbsp;&nbsp;&nbsp;&nbsp; A list of words to remove.

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

## Text Analyzers

The text analyzer combines the tokenizer with some filters and uses it to parse the text of the field.  
For example, write as follows:

```json
{
  "lang_en": {
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
          "stemmer_algorithm": "english"
        }
      },
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
    ]
  }
}
```

The field uses the above text analyzer are described as follows:

```json
[
  {
    "name": "description",
    "type": "text",
    "options": {
      "indexing": {
        "record": "position",
        "tokenizer": "lang_en"
      },
      "stored": true
    }
  }
]
```
