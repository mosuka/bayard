use log::*;
use serde_json::Value;
use tantivy::tokenizer::NgramTokenizer;

#[derive(Clone)]
pub struct NgramTokenizerFactory {}

impl NgramTokenizerFactory {
    pub fn new() -> Self {
        NgramTokenizerFactory {}
    }

    pub fn create(self, json: &str) -> NgramTokenizer {
        let v: Value = match serde_json::from_str(json) {
            Result::Ok(val) => val,
            Result::Err(err) => {
                warn!("failed to parse JSON: {}", err.to_string());
                serde_json::Value::Null
            }
        };

        let min_gram: usize;
        match v["min_gram"].as_f64() {
            Some(l) => {
                min_gram = l as usize;
            }
            _ => {
                warn!("min_gram is missing. set 1 as default");
                min_gram = 1 as usize;
            }
        }

        let max_gram: usize;
        match v["max_gram"].as_f64() {
            Some(l) => {
                max_gram = l as usize;
            }
            _ => {
                warn!("max_gram is missing. set 1 as default");
                max_gram = 1 as usize;
            }
        }

        let prefix_only: bool;
        match v["prefix_only"].as_bool() {
            Some(l) => {
                prefix_only = l;
            }
            _ => {
                warn!("prefix_only is missing. set false as default");
                prefix_only = false;
            }
        }

        NgramTokenizer::new(min_gram, max_gram, prefix_only)
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::Tokenizer;

    use crate::tokenizer::ngram_tokenizer_factory::NgramTokenizerFactory;

    #[test]
    fn test_ngram_tokenizer() {
        let json = r#"
            {
                "min_gram": 2,
                "max_gram": 3,
                "prefix_only": false
            }
        "#;

        let factory = NgramTokenizerFactory::new();
        let tokenizer = factory.create(json);

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
}
