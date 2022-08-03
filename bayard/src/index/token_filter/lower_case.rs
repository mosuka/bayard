use tantivy::tokenizer::{BoxTokenFilter, LowerCaser};

pub fn create_lower_case_filter() -> BoxTokenFilter {
    BoxTokenFilter::from(LowerCaser {})
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::index::token_filter::lower_case::create_lower_case_filter;

    fn helper(text: &str) -> Vec<String> {
        let filter = create_lower_case_filter();

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
    fn test_lower_case_filter() {
        assert_eq!(vec!["phalanx".to_string()], helper("Phalanx"));
    }
}
