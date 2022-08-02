use tantivy::tokenizer::{AsciiFoldingFilter, BoxTokenFilter};

pub fn create_ascii_folding_filter() -> BoxTokenFilter {
    BoxTokenFilter::from(AsciiFoldingFilter {})
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::index::token_filter::ascii_folding::create_ascii_folding_filter;

    fn helper(text: &str) -> Vec<String> {
        let filter = create_ascii_folding_filter();

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
    fn test_ascii_folding_filter() {
        assert_eq!(vec!["Ramon".to_string()], helper("Rámon"));
    }
}
