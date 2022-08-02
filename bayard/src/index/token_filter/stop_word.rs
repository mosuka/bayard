use tantivy::tokenizer::{BoxTokenFilter, StopWordFilter};

use crate::index::token_filter::{FilterFactoryError, FilterFactoryErrorKind};

const DEFAULT_STOP_WORDS: Vec<String> = vec![];

#[derive(Deserialize, Serialize)]
struct StopWordFilterArgs {
    words: Vec<String>,
}

pub fn create_stop_word_filter(args: &[u8]) -> Result<BoxTokenFilter, FilterFactoryError> {
    if args.is_empty() {
        return Ok(BoxTokenFilter::from(StopWordFilter::remove(
            DEFAULT_STOP_WORDS,
        )));
    }

    let stop_word_args = serde_json::from_slice::<StopWordFilterArgs>(args)
        .map_err(|e| FilterFactoryErrorKind::InvalidArgument.with_error(e))?;

    Ok(BoxTokenFilter::from(StopWordFilter::remove(
        stop_word_args.words,
    )))
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::index::token_filter::stop_word::create_stop_word_filter;

    fn helper(text: &str) -> Vec<String> {
        let json_str = r#"
            {
                "words": [
                    "a",
                    "b",
                    "c"
                ]
            }
            "#;
        let json = json_str.as_bytes();

        let filter = create_stop_word_filter(json).unwrap();

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
