use std::sync::Arc;

use cang_jie::{CangJieTokenizer, TokenizerOption};
use jieba_rs::Jieba;
use log::*;
use serde_json::Value;

#[derive(Clone)]
pub struct CangJieTokenizerFactory {}

impl CangJieTokenizerFactory {
    pub fn new() -> Self {
        CangJieTokenizerFactory {}
    }

    pub fn create(self, json: &str) -> CangJieTokenizer {
        let v: Value = match serde_json::from_str(json) {
            Result::Ok(val) => val,
            Result::Err(err) => {
                warn!("failed to parse JSON: {}", err.to_string());
                serde_json::Value::Null
            }
        };

        let hmm: bool;
        match v["hmm"].as_bool() {
            Some(l) => {
                hmm = l;
            }
            _ => {
                warn!("hmm is missing. set false as default");
                hmm = false;
            }
        }

        let tokenizer_option;
        match v["tokenizer_option"].as_str() {
            Some(opt) => match opt {
                "all" => tokenizer_option = TokenizerOption::All,
                "default" => tokenizer_option = TokenizerOption::Default { hmm },
                "search" => tokenizer_option = TokenizerOption::ForSearch { hmm },
                "unicode" => tokenizer_option = TokenizerOption::Unicode,
                _ => {
                    tokenizer_option = TokenizerOption::Default { hmm };
                }
            },
            _ => {
                warn!("tokenizer_option is missing. set \"Default\" as default");
                tokenizer_option = TokenizerOption::Default { hmm };
            }
        }

        CangJieTokenizer {
            worker: Arc::new(Jieba::default()),
            option: tokenizer_option,
        }
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::Tokenizer;

    use crate::tokenizer::cang_jie_tokenizer_factory::CangJieTokenizerFactory;

    #[test]
    fn test_cang_jie_tokenizer() {
        let json = r#"
            {
                "hmm": false,
                "tokenizer_option": "default"
            }
        "#;

        let factory = CangJieTokenizerFactory::new();
        let tokenizer = factory.create(json);

        let mut stream = tokenizer.token_stream("我们中出了一个叛徒");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "我们");
            assert_eq!(token.offset_from, 6);
            assert_eq!(token.offset_to, 12);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "中");
            assert_eq!(token.offset_from, 9);
            assert_eq!(token.offset_to, 12);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "出");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 15);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "了");
            assert_eq!(token.offset_from, 15);
            assert_eq!(token.offset_to, 18);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "一个");
            assert_eq!(token.offset_from, 21);
            assert_eq!(token.offset_to, 27);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "叛徒");
            assert_eq!(token.offset_from, 27);
            assert_eq!(token.offset_to, 33);
        }
        assert!(stream.next().is_none());
    }
}
