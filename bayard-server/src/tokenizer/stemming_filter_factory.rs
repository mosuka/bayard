use log::*;
use serde_json::Value;
use tantivy::tokenizer::{Language, Stemmer};

#[derive(Clone)]
pub struct StemmingFilterFactory {}

impl StemmingFilterFactory {
    pub fn new() -> Self {
        StemmingFilterFactory {}
    }

    pub fn create(self, json: &str) -> Stemmer {
        let v: Value = match serde_json::from_str(json) {
            Result::Ok(val) => val,
            Result::Err(err) => {
                warn!("failed to parse JSON: {}", err.to_string());
                serde_json::Value::Null
            }
        };

        let stemmer_algorithm: Language;
        match v["stemmer_algorithm"].as_str() {
            Some(l) => match l {
                "arabic" => stemmer_algorithm = Language::Arabic,
                "danish" => stemmer_algorithm = Language::Danish,
                "dutch" => stemmer_algorithm = Language::Dutch,
                "english" => stemmer_algorithm = Language::English,
                "finnish" => stemmer_algorithm = Language::Finnish,
                "french" => stemmer_algorithm = Language::French,
                "german" => stemmer_algorithm = Language::German,
                "greek" => stemmer_algorithm = Language::Greek,
                "hungarian" => stemmer_algorithm = Language::Hungarian,
                "italian" => stemmer_algorithm = Language::Italian,
                "norwegian" => stemmer_algorithm = Language::Norwegian,
                "portuguese" => stemmer_algorithm = Language::Portuguese,
                "romanian" => stemmer_algorithm = Language::Romanian,
                "russian" => stemmer_algorithm = Language::Russian,
                "spanish" => stemmer_algorithm = Language::Spanish,
                "swedish" => stemmer_algorithm = Language::Swedish,
                "tamil" => stemmer_algorithm = Language::Tamil,
                "turkish" => stemmer_algorithm = Language::Turkish,
                _ => {
                    stemmer_algorithm = Language::English;
                }
            },
            _ => {
                warn!("stemmer_algorithm is missing. set \"English\" as default");
                stemmer_algorithm = Language::English;
            }
        }

        Stemmer::new(stemmer_algorithm)
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::tokenizer::stemming_filter_factory::StemmingFilterFactory;

    fn helper(text: &str) -> Vec<String> {
        let json = r#"
            {
                "stemmer_algorithm": "french"
            }
            "#;

        let factory = StemmingFilterFactory::new();
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
                "continu".to_string(),
                "continu".to_string(),
                "continu".to_string(),
                "continu".to_string(),
                "continu".to_string(),
                "continu".to_string(),
                "continu".to_string(),
            ],
            helper("continu continua continuait continuant continuation continue continué")
        );
    }
}
