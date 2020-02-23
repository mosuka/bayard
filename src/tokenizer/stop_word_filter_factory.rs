use log::*;
use serde_json::Value;
use tantivy::tokenizer::StopWordFilter;

#[derive(Clone)]
pub struct StopWordFilterFactory {}

impl StopWordFilterFactory {
    pub fn new() -> Self {
        StopWordFilterFactory {}
    }

    pub fn create(self, json: &str) -> StopWordFilter {
        let v: Value = match serde_json::from_str(json) {
            Result::Ok(val) => val,
            Result::Err(err) => {
                warn!("failed to parse JSON: {}", err.to_string());
                serde_json::Value::Null
            }
        };

        match v["words"].as_array() {
            Some(w) => {
                if w.len() > 0 {
                    StopWordFilter::remove(
                        w.iter().map(|s| s.as_str().unwrap().to_string()).collect(),
                    )
                } else {
                    warn!("words are empty. set default words");
                    StopWordFilter::default()
                }
            }
            _ => {
                warn!("words are missing. set default words");
                StopWordFilter::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::tokenizer::stop_word_filter_factory::StopWordFilterFactory;

    fn helper(text: &str) -> Vec<String> {
        let json = r#"
            {
                "words": [
                    "a",
                    "b",
                    "c"
                ]
            }
            "#;

        let factory = StopWordFilterFactory::new();
        let filter = factory.create(json);

        let mut tokens = vec![];
        let mut token_stream = TextAnalyzer::from(SimpleTokenizer)
            .filter(filter)
            .token_stream(text);
        while token_stream.advance() {
            let token_text = token_stream.token().text.clone();
            tokens.push(token_text);
        }
        tokens
    }

    #[test]
    fn test_stemming_filter() {
        assert_eq!(
            vec![
                "d".to_string(),
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
            ],
            helper("a b c d e f g")
        );
    }
}
