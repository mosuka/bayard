use lindera_tantivy::tokenizer::LinderaTokenizer;
use log::*;
use serde_json::Value;

#[derive(Clone)]
pub struct LinderaTokenizerFactory {}

impl LinderaTokenizerFactory {
    pub fn new() -> Self {
        LinderaTokenizerFactory {}
    }

    pub fn create(self, json: &str) -> LinderaTokenizer {
        let v: Value = match serde_json::from_str(json) {
            Result::Ok(val) => val,
            Result::Err(err) => {
                warn!("failed to parse JSON: {}", err.to_string());
                serde_json::Value::Null
            }
        };

        let mode: &str;
        match v["mode"].as_str() {
            Some(m) => {
                mode = m;
            }
            _ => {
                warn!("mode is missing. set `normal` as default");
                mode = "normal";
            }
        }

        let dict: &str;
        match v["dict"].as_str() {
            Some(d) => {
                dict = d;
            }
            _ => {
                warn!("dict is missing. set `` as default");
                dict = "";
            }
        }

        LinderaTokenizer::new(mode, dict)
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::Tokenizer;

    use crate::tokenizer::lindera_tokenizer_factory::LinderaTokenizerFactory;

    #[test]
    fn test_ngram_tokenizer() {
        let json = r#"
            {
                "mode": "normal"
            }
        "#;

        let factory = LinderaTokenizerFactory::new();
        let tokenizer = factory.create(json);

        let mut stream = tokenizer.token_stream("すもももももももものうち");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "すもも");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 9);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "も");
            assert_eq!(token.offset_from, 9);
            assert_eq!(token.offset_to, 12);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "もも");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 18);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "も");
            assert_eq!(token.offset_from, 18);
            assert_eq!(token.offset_to, 21);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "もも");
            assert_eq!(token.offset_from, 21);
            assert_eq!(token.offset_to, 27);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "の");
            assert_eq!(token.offset_from, 27);
            assert_eq!(token.offset_to, 30);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "うち");
            assert_eq!(token.offset_from, 30);
            assert_eq!(token.offset_to, 36);
        }
        assert!(stream.next().is_none());
    }
}
