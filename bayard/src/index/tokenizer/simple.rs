use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};

pub fn create_simple_analyzer() -> TextAnalyzer {
    TextAnalyzer::from(SimpleTokenizer {})
}

#[cfg(test)]
mod tests {
    use crate::index::tokenizer::simple::create_simple_analyzer;

    #[test]
    fn test_simple_tokenizer() {
        let tokenizer = create_simple_analyzer();

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
