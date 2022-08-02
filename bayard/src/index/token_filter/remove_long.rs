use tantivy::tokenizer::{BoxTokenFilter, RemoveLongFilter};

use crate::index::token_filter::{FilterFactoryError, FilterFactoryErrorKind};

// https://en.wikipedia.org/wiki/Donaudampfschiffahrtselektrizit%C3%A4tenhauptbetriebswerkbauunterbeamtengesellschaft
const DEFAULT_LENGTH_LIMIT: usize = 80;

#[derive(Deserialize, Serialize)]
struct RemoveLongFilterArgs {
    length_limit: usize,
}

pub fn create_remove_long_filter(args: &[u8]) -> Result<BoxTokenFilter, FilterFactoryError> {
    if args.is_empty() {
        return Ok(BoxTokenFilter::from(RemoveLongFilter::limit(
            DEFAULT_LENGTH_LIMIT,
        )));
    }

    let remove_long_args = serde_json::from_slice::<RemoveLongFilterArgs>(args)
        .map_err(|e| FilterFactoryErrorKind::InvalidArgument.with_error(e))?;

    Ok(BoxTokenFilter::from(RemoveLongFilter::limit(
        remove_long_args.length_limit,
    )))
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::index::token_filter::remove_long::create_remove_long_filter;

    fn helper(text: &str) -> Vec<String> {
        let json_str = r#"
        {
            "length_limit": 10
        }
        "#;
        let json = json_str.as_bytes();

        let filter = create_remove_long_filter(json).unwrap();

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
