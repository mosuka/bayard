use tantivy::tokenizer::AlphaNumOnlyFilter;

#[derive(Clone)]
pub struct AlphaNumOnlyFilterFactory {}

impl AlphaNumOnlyFilterFactory {
    pub fn new() -> Self {
        AlphaNumOnlyFilterFactory {}
    }

    pub fn create(self) -> AlphaNumOnlyFilter {
        AlphaNumOnlyFilter {}
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::{SimpleTokenizer, TokenStream, Tokenizer};

    use crate::tokenizer::alpha_num_only_filter_factory::AlphaNumOnlyFilterFactory;

    fn helper(text: &str) -> Vec<String> {
        let factory = AlphaNumOnlyFilterFactory::new();
        let filter = factory.create();

        let mut tokens = vec![];
        let mut token_stream = SimpleTokenizer.filter(filter).token_stream(text);
        while token_stream.advance() {
            let token_text = token_stream.token().text.clone();
            tokens.push(token_text);
        }
        tokens
    }

    #[test]
    fn test_alpha_num_only_filter() {
        assert_eq!(
            vec!["bayard".to_string(), "search".to_string()],
            helper("bayard-search")
        );
    }
}
