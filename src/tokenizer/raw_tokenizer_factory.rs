use tantivy::tokenizer::RawTokenizer;

#[derive(Clone)]
pub struct RawTokenizerFactory {}

impl RawTokenizerFactory {
    pub fn new() -> Self {
        RawTokenizerFactory {}
    }

    pub fn create(self) -> RawTokenizer {
        RawTokenizer {}
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::Tokenizer;

    use crate::tokenizer::raw_tokenizer_factory::RawTokenizerFactory;

    #[test]
    fn test_raw_tokenizer() {
        let factory = RawTokenizerFactory::new();
        let tokenizer = factory.create();

        let mut stream = tokenizer.token_stream("hello");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hello");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 5);
        }
        assert!(stream.next().is_none());
    }
}
