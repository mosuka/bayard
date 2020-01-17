# Pre-configured tokenizers

Bayard provides some pre-configured tokenizers.

## General tokenizers

Tokenizers are responsible for breaking field data into lexical units, or tokens.

### raw

This tokenizer treats the entire text field as a single token.

### default

Tokenize the text by splitting on whitespace and punctuation. Tokens will be removed that are longer than 40 bytes (in UTF-8 representation), and they will be changed to lowercase terms.

### unigram

Reads the text and generates unigram tokens. Then tokens will be changed to lowercase terms.

### bigram

Reads the text and generates bigram tokens. Then tokens will be changed to lowercase terms.

### trigram

Reads the text and generates trigram tokens. Then tokens will be changed to lowercase terms.

## Language specific tokenizers

These factories are each designed to work with specific languages. The languages covered here are:

### lang_ar

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Arabic stemmer will applied.

### lang_da

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Danish stemmer will applied.

### lang_de

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that German stemmer will applied.

### lang_el

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Greek stemmer will applied.

### lang_en

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that English stemmer will applied.

### lang_es

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Spanish stemmer will applied.

### lang_fi

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Finnish stemmer will applied.

### lang_fr

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that French stemmer will applied.

### lang_hu

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Hungarian stemmer will applied.

### lang_it

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Italian stemmer will applied.

### lang_nl

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Dutch stemmer will applied.

### lang_no

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Norwegian stemmer will applied.

### lang_pt

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Portuguese stemmer will applied.

### lang_ro

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Romanian stemmer will applied.

### lang_ru

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Russian stemmer will applied.

### lang_sv

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Swedish stemmer will applied.

### lang_ta

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Tamil stemmer will applied.

### lang_tr

It basically behaves the same as the `default` tokenizer. Additionally converts alphabetic, numeric, and symbolic Unicode characters which are not in the first 127 ASCII characters (the "Basic Latin" Unicode block) into their ASCII equivalents, if one exists. After that Turkish stemmer will applied.

### lang_zh

Tokenize the text by using embedded chinese dictionary and splitting on whitespace and punctuation. Tokens will be removed that are longer than 40 bytes (in UTF-8 representation), and they will be changed to lowercase terms.
