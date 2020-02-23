use tantivy::tokenizer::AsciiFoldingFilter;

#[derive(Clone)]
pub struct AsciiFoldingFilterFactory {}

impl AsciiFoldingFilterFactory {
    pub fn new() -> Self {
        AsciiFoldingFilterFactory {}
    }

    pub fn create(self) -> AsciiFoldingFilter {
        AsciiFoldingFilter {}
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::tokenizer::ascii_folding_filter_factory::AsciiFoldingFilterFactory;

    fn helper(text: &str) -> Vec<String> {
        let factory = AsciiFoldingFilterFactory::new();
        let filter = factory.create();

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
