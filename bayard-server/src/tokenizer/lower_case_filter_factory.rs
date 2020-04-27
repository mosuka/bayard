use tantivy::tokenizer::LowerCaser;

#[derive(Clone)]
pub struct LowerCaseFilterFactory {}

impl LowerCaseFilterFactory {
    pub fn new() -> Self {
        LowerCaseFilterFactory {}
    }

    pub fn create(self) -> LowerCaser {
        LowerCaser {}
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

    use crate::tokenizer::lower_case_filter_factory::LowerCaseFilterFactory;

    fn helper(text: &str) -> Vec<String> {
        let factory = LowerCaseFilterFactory::new();
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
    fn test_lower_case_filter() {
        assert_eq!(vec!["bayard".to_string()], helper("Bayard"));
    }
}
