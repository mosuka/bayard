use log::*;
use serde_json::Value;
use tantivy::tokenizer::RemoveLongFilter;

#[derive(Clone)]
pub struct RemoveLongFilterFactory {}

impl RemoveLongFilterFactory {
    pub fn new() -> Self {
        RemoveLongFilterFactory {}
    }

    pub fn create(self, json: &str) -> RemoveLongFilter {
        let v: Value = match serde_json::from_str(json) {
            Result::Ok(val) => val,
            Result::Err(err) => {
                warn!("failed to parse JSON: {}", err.to_string());
                serde_json::Value::Null
            }
        };

        let length_limit: usize;
        match v["length_limit"].as_f64() {
            Some(l) => {
                length_limit = l as usize;
            }
            _ => {
                // floccinaucinihilipilification
                // supercalifragilisticexpialidocious
                // pneumonoultramicroscopicsilicovolcanoconiosis
                // Donaudampfschiffahrtselektrizitätenhauptbetriebswerkbauunterbeamtengesellschaft
                warn!("length_limit is missing. set 80 as default");
                length_limit = 80 as usize;
            }
        }

        RemoveLongFilter::limit(length_limit)
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::tokenizer::remove_long_filter_factory::RemoveLongFilterFactory;

    fn helper(text: &str) -> Vec<String> {
        let json = r#"
            {
                "length_limit": 10
            }
        "#;

        let factory = RemoveLongFilterFactory::new();
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
    fn test_remove_long_filter() {
        assert_eq!(
            vec![
                "1".to_string(),
                "12".to_string(),
                "123".to_string(),
                "1234".to_string(),
                "12345".to_string(),
                "123456".to_string(),
                "1234567".to_string(),
                "12345678".to_string(),
                "123456789".to_string(),
            ],
            helper("1 12 123 1234 12345 123456 1234567 12345678 123456789 1234567890 12345678901")
        );
    }
}
