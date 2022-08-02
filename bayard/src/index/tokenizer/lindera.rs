use lindera_tantivy::tokenizer::{LinderaTokenizer, TokenizerConfig};
use tantivy::tokenizer::TextAnalyzer;

use crate::index::tokenizer::{TokenizerFactoryError, TokenizerFactoryErrorKind};

type LinderaTokenizerArgs = TokenizerConfig;

pub fn create_lindera_analyzer(args: &[u8]) -> Result<TextAnalyzer, TokenizerFactoryError> {
    if args.is_empty() {
        return Ok(TextAnalyzer::from(LinderaTokenizer::new().map_err(
            |e| TokenizerFactoryErrorKind::CreateError.with_error(e),
        )?));
    }

    let lindera_args = serde_json::from_slice::<LinderaTokenizerArgs>(args)
        .map_err(|e| TokenizerFactoryErrorKind::InvalidArgument.with_error(e))?;

    Ok(TextAnalyzer::from(
        LinderaTokenizer::with_config(lindera_args)
            .map_err(|e| TokenizerFactoryErrorKind::CreateError.with_error(e))?,
    ))
}

#[cfg(test)]
mod tests {
    use lindera_tantivy::{
        mode::{Mode, Penalty},
        tokenizer::DictionaryKind,
    };

    use crate::index::tokenizer::lindera::{create_lindera_analyzer, LinderaTokenizerArgs};

    #[test]
    #[cfg(any(
        feature = "ipadic",
        feature = "unidic",
        feature = "ko-dic",
        feature = "cc-cedict"
    ))]
    fn test_from_slice() {
        let json_str = r#"
        {
            "dictionary": {
                "kind": "ipadic"
            },
            "mode": {
                "decompose": {
                    "kanji_penalty_length_threshold": 2,
                    "kanji_penalty_length_penalty": 3000,
                    "other_penalty_length_threshold": 7,
                    "other_penalty_length_penalty": 1700
                }
            }
        }
        "#;
        let json = json_str.as_bytes();

        let args = serde_json::from_slice::<LinderaTokenizerArgs>(json).unwrap();
        assert_eq!(args.dictionary.kind, DictionaryKind::IPADIC);
        // assert_eq!(args.user_dictionary, None);
        assert_eq!(args.mode, Mode::Decompose(Penalty::default()));
    }

    #[test]
    #[cfg(feature = "ipadic")]
    fn test_lindera_ipadic_tokenizer() {
        let json_str = r#"
        {
            "dictionary": {
                "kind": "ipadic"
            },
            "mode": "normal"
        }
        "#;
        let json = json_str.as_bytes();

        let tokenizer = create_lindera_analyzer(json).unwrap();

        let mut stream = tokenizer.token_stream("日本語の形態素解析を行うことができます。");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "日本語");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 9);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "の");
            assert_eq!(token.offset_from, 9);
            assert_eq!(token.offset_to, 12);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "形態素");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 21);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "解析");
            assert_eq!(token.offset_from, 21);
            assert_eq!(token.offset_to, 27);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "を");
            assert_eq!(token.offset_from, 27);
            assert_eq!(token.offset_to, 30);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "行う");
            assert_eq!(token.offset_from, 30);
            assert_eq!(token.offset_to, 36);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "こと");
            assert_eq!(token.offset_from, 36);
            assert_eq!(token.offset_to, 42);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "が");
            assert_eq!(token.offset_from, 42);
            assert_eq!(token.offset_to, 45);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "でき");
            assert_eq!(token.offset_from, 45);
            assert_eq!(token.offset_to, 51);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "ます");
            assert_eq!(token.offset_from, 51);
            assert_eq!(token.offset_to, 57);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "。");
            assert_eq!(token.offset_from, 57);
            assert_eq!(token.offset_to, 60);
        }
        assert!(stream.next().is_none());
    }

    #[test]
    #[cfg(feature = "unidic")]
    fn test_lindera_unidic_tokenizer() {
        let json_str = r#"
        {
            "dictionary": {
                "kind": "ipadic"
            },
            "mode": "normal"
        }
        "#;
        let json = json_str.as_bytes();

        let tokenizer = create_lindera_analyzer(json).unwrap();

        let mut stream = tokenizer.token_stream("日本語の形態素解析を行うことができます。");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "日本");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 6);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "語");
            assert_eq!(token.offset_from, 6);
            assert_eq!(token.offset_to, 9);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "の");
            assert_eq!(token.offset_from, 9);
            assert_eq!(token.offset_to, 12);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "形態");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 18);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "素");
            assert_eq!(token.offset_from, 18);
            assert_eq!(token.offset_to, 21);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "解析");
            assert_eq!(token.offset_from, 21);
            assert_eq!(token.offset_to, 27);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "を");
            assert_eq!(token.offset_from, 27);
            assert_eq!(token.offset_to, 30);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "行う");
            assert_eq!(token.offset_from, 30);
            assert_eq!(token.offset_to, 36);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "こと");
            assert_eq!(token.offset_from, 36);
            assert_eq!(token.offset_to, 42);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "が");
            assert_eq!(token.offset_from, 42);
            assert_eq!(token.offset_to, 45);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "でき");
            assert_eq!(token.offset_from, 45);
            assert_eq!(token.offset_to, 51);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "ます");
            assert_eq!(token.offset_from, 51);
            assert_eq!(token.offset_to, 57);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "。");
            assert_eq!(token.offset_from, 57);
            assert_eq!(token.offset_to, 60);
        }
        assert!(stream.next().is_none());
    }

    #[test]
    #[cfg(feature = "ko-dic")]
    fn test_lindera_kodic_tokenizer() {
        let json_str = r#"
        {
            "dictionary": {
                "kind": "ko-dic"
            },
            "mode": "normal"
        }
        "#;
        let json = json_str.as_bytes();

        let tokenizer = create_lindera_analyzer(json).unwrap();

        let mut stream = tokenizer.token_stream("한국어의형태해석을실시할수있습니다.");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "한국어");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 9);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "의");
            assert_eq!(token.offset_from, 9);
            assert_eq!(token.offset_to, 12);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "형태");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 18);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "해석");
            assert_eq!(token.offset_from, 18);
            assert_eq!(token.offset_to, 24);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "을");
            assert_eq!(token.offset_from, 24);
            assert_eq!(token.offset_to, 27);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "실시");
            assert_eq!(token.offset_from, 27);
            assert_eq!(token.offset_to, 33);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "할");
            assert_eq!(token.offset_from, 33);
            assert_eq!(token.offset_to, 36);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "수");
            assert_eq!(token.offset_from, 36);
            assert_eq!(token.offset_to, 39);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "있");
            assert_eq!(token.offset_from, 39);
            assert_eq!(token.offset_to, 42);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "습니다");
            assert_eq!(token.offset_from, 42);
            assert_eq!(token.offset_to, 51);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, ".");
            assert_eq!(token.offset_from, 51);
            assert_eq!(token.offset_to, 52);
        }
        assert!(stream.next().is_none());
    }

    #[test]
    #[cfg(feature = "cc-cedict")]
    fn test_lindera_cedict_tokenizer() {
        let json_str = r#"
        {
            "dictionary": {
                "kind": "cc-cedict"
            },
            "mode": "normal"
        }
        "#;
        let json = json_str.as_bytes();

        let tokenizer = create_lindera_analyzer(json).unwrap();

        let mut stream = tokenizer.token_stream("可以进行中文形态学分析。");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "可以");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 6);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "进行");
            assert_eq!(token.offset_from, 6);
            assert_eq!(token.offset_to, 12);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "中文");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 18);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "形态学");
            assert_eq!(token.offset_from, 18);
            assert_eq!(token.offset_to, 27);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "分析");
            assert_eq!(token.offset_from, 27);
            assert_eq!(token.offset_to, 33);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "。");
            assert_eq!(token.offset_from, 33);
            assert_eq!(token.offset_to, 36);
        }
        assert!(stream.next().is_none());
    }
}
