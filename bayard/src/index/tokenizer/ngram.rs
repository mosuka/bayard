use tantivy::tokenizer::{NgramTokenizer, TextAnalyzer};

use crate::index::tokenizer::{TokenizerFactoryError, TokenizerFactoryErrorKind};

const DEFAULT_MIN_GRAM: usize = 1;
const DEFAULT_MAX_GRAM: usize = 2;
const DEFAULT_PREFIX_ONLY: bool = false;

#[derive(Deserialize, Serialize)]
struct NgramTokenizerArgs {
    min_gram: usize,
    max_gram: usize,
    prefix_only: bool,
}

pub fn create_ngram_analyzer(args: &[u8]) -> Result<TextAnalyzer, TokenizerFactoryError> {
    if args.is_empty() {
        return Ok(TextAnalyzer::from(NgramTokenizer::new(
            DEFAULT_MIN_GRAM,
            DEFAULT_MAX_GRAM,
            DEFAULT_PREFIX_ONLY,
        )));
    }

    let ngram_args = serde_json::from_slice::<NgramTokenizerArgs>(args)
        .map_err(|e| TokenizerFactoryErrorKind::InvalidArgument.with_error(e))?;

    Ok(TextAnalyzer::from(NgramTokenizer::new(
        ngram_args.min_gram,
        ngram_args.max_gram,
        ngram_args.prefix_only,
    )))
}

#[cfg(test)]
mod tests {
    use crate::index::tokenizer::ngram::{create_ngram_analyzer, NgramTokenizerArgs};

    #[test]
    fn test_from_slice() {
        let json_str = r#"
        {
            "min_gram": 2,
            "max_gram": 3,
            "prefix_only": false
        }
        "#;
        let json = json_str.as_bytes();

        let args = serde_json::from_slice::<NgramTokenizerArgs>(json).unwrap();
        assert_eq!(args.min_gram, 2);
        assert_eq!(args.max_gram, 3);
        assert_eq!(args.prefix_only, false);
    }

    #[test]
    fn test_ngram_tokenizer() {
        let json_str = r#"
        {
            "min_gram": 2,
            "max_gram": 3,
            "prefix_only": false
        }
        "#;
        let json = json_str.as_bytes();

        let tokenizer = create_ngram_analyzer(json).unwrap();

        let mut stream = tokenizer.token_stream("hello");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "he");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 2);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hel");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 3);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "el");
            assert_eq!(token.offset_from, 1);
            assert_eq!(token.offset_to, 3);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "ell");
            assert_eq!(token.offset_from, 1);
            assert_eq!(token.offset_to, 4);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "ll");
            assert_eq!(token.offset_from, 2);
            assert_eq!(token.offset_to, 4);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "llo");
            assert_eq!(token.offset_from, 2);
            assert_eq!(token.offset_to, 5);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "lo");
            assert_eq!(token.offset_from, 3);
            assert_eq!(token.offset_to, 5);
        }
        assert!(stream.next().is_none());
    }

    #[test]
    fn test_ngram_tokenizer_prefix_only() {
        let json_str = r#"
        {
            "min_gram": 2,
            "max_gram": 5,
            "prefix_only": true
        }
        "#;
        let json = json_str.as_bytes();

        let tokenizer = create_ngram_analyzer(json).unwrap();

        let mut stream = tokenizer.token_stream("hello");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "he");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 2);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hel");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 3);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hell");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 4);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hello");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 5);
        }
        assert!(stream.next().is_none());
    }
}
