use tantivy::tokenizer::{TextAnalyzer, WhitespaceTokenizer};

pub fn create_whitespace_analyzer() -> TextAnalyzer {
    TextAnalyzer::from(WhitespaceTokenizer {})
}

#[cfg(test)]
mod tests {
    use crate::index::tokenizer::whitespace::create_whitespace_analyzer;

    #[test]
    fn test_whitespace_tokenizer() {
        let tokenizer = create_whitespace_analyzer();

        let mut stream = tokenizer.token_stream("hello world!");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hello");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 5);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "world!");
            assert_eq!(token.offset_from, 6);
            assert_eq!(token.offset_to, 12);
        }
        assert!(stream.next().is_none());
    }
}
