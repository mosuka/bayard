use tantivy::tokenizer::{BoxTokenFilter, Language, Stemmer};

use crate::index::token_filter::{FilterFactoryError, FilterFactoryErrorKind};

const DEFAULT_LANGUAGE: Language = Language::English;

#[derive(Deserialize, Serialize)]
struct StemmingFilterArgs {
    language: Language,
}

pub fn create_stemming_filter(args: &[u8]) -> Result<BoxTokenFilter, FilterFactoryError> {
    if args.is_empty() {
        return Ok(BoxTokenFilter::from(Stemmer::new(DEFAULT_LANGUAGE)));
    }

    let stemming_args = serde_json::from_slice::<StemmingFilterArgs>(args)
        .map_err(|e| FilterFactoryErrorKind::InvalidArgument.with_error(e))?;

    Ok(BoxTokenFilter::from(Stemmer::new(stemming_args.language)))
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::index::token_filter::stemming::create_stemming_filter;

    fn helper(text: &str) -> Vec<String> {
        let json_str = r#"
            {
                "language": "French"
            }
            "#;
        let json = json_str.as_bytes();

        let filter = create_stemming_filter(json).unwrap();

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
