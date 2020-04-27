use tantivy::tokenizer::FacetTokenizer;

#[derive(Clone)]
pub struct FacetTokenizerFactory {}

impl FacetTokenizerFactory {
    pub fn new() -> Self {
        FacetTokenizerFactory {}
    }

    pub fn create(self) -> FacetTokenizer {
        FacetTokenizer {}
    }
}

#[cfg(test)]
mod tests {
    use tantivy::schema::Facet;
    use tantivy::tokenizer::{Token, Tokenizer};

    use crate::tokenizer::facet_tokenizer_factory::FacetTokenizerFactory;

    #[test]
    fn test_facet_tokenizer() {
        let factory = FacetTokenizerFactory::new();
        let tokenizer = factory.create();

        let facet = Facet::from_path(vec!["top", "a", "b"]);

        let mut tokens = vec![];
        {
            let mut add_token = |token: &Token| {
                let facet = Facet::from_encoded(token.text.as_bytes().to_owned()).unwrap();
                tokens.push(format!("{}", facet));
            };

            tokenizer
                .token_stream(facet.encoded_str())
                .process(&mut add_token);
        }

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], "/");
        assert_eq!(tokens[1], "/top");
        assert_eq!(tokens[2], "/top/a");
        assert_eq!(tokens[3], "/top/a/b");
    }
}
