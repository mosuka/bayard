use tantivy::tokenizer::{RawTokenizer, TextAnalyzer};

pub fn create_raw_analyzer() -> TextAnalyzer {
    TextAnalyzer::from(RawTokenizer {})
}

#[cfg(test)]
mod tests {
    use crate::index::tokenizer::raw::create_raw_analyzer;

    #[test]
    fn test_raw_tokenizer() {
        let tokenizer = create_raw_analyzer();

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
