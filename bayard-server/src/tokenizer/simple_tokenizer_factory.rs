use tantivy::tokenizer::SimpleTokenizer;

#[derive(Clone)]
pub struct SimpleTokenizerFactory {}

impl SimpleTokenizerFactory {
    pub fn new() -> Self {
        SimpleTokenizerFactory {}
    }

    pub fn create(self) -> SimpleTokenizer {
        SimpleTokenizer {}
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::Tokenizer;

    use crate::tokenizer::simple_tokenizer_factory::SimpleTokenizerFactory;

    #[test]
    fn test_simple_tokenizer() {
        let factory = SimpleTokenizerFactory::new();
        let tokenizer = factory.create();

        let mut stream = tokenizer.token_stream("hello world!");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hello");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 5);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "world");
            assert_eq!(token.offset_from, 6);
            assert_eq!(token.offset_to, 11);
        }
        assert!(stream.next().is_none());
    }
}
