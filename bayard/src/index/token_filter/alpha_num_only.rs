use tantivy::tokenizer::{AlphaNumOnlyFilter, BoxTokenFilter};

pub fn create_alpha_num_only_filter() -> BoxTokenFilter {
    BoxTokenFilter::from(AlphaNumOnlyFilter {})
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::index::token_filter::alpha_num_only::create_alpha_num_only_filter;

    fn helper(text: &str) -> Vec<String> {
        let filter = create_alpha_num_only_filter();

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
    fn test_alpha_num_only_filter() {
        assert_eq!(
            vec!["parabuteo".to_string(), "123".to_string()],
            helper("parabuteo-123")
        );
    }
}
