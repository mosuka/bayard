use log::*;
use serde_json::Value;
use tantivy::tokenizer::{BoxTokenFilter, TextAnalyzer, TokenizerManager};

use crate::tokenizer::alpha_num_only_filter_factory::AlphaNumOnlyFilterFactory;
use crate::tokenizer::ascii_folding_filter_factory::AsciiFoldingFilterFactory;
use crate::tokenizer::cang_jie_tokenizer_factory::CangJieTokenizerFactory;
use crate::tokenizer::facet_tokenizer_factory::FacetTokenizerFactory;
use crate::tokenizer::lindera_tokenizer_factory::LinderaTokenizerFactory;
use crate::tokenizer::lower_case_filter_factory::LowerCaseFilterFactory;
use crate::tokenizer::ngram_tokenizer_factory::NgramTokenizerFactory;
use crate::tokenizer::raw_tokenizer_factory::RawTokenizerFactory;
use crate::tokenizer::remove_long_filter_factory::RemoveLongFilterFactory;
use crate::tokenizer::simple_tokenizer_factory::SimpleTokenizerFactory;
use crate::tokenizer::stemming_filter_factory::StemmingFilterFactory;
use crate::tokenizer::stop_word_filter_factory::StopWordFilterFactory;

pub struct TokenizerInitializer {
    facet_tokenizer_factory: FacetTokenizerFactory,
    ngram_tokenizer_factory: NgramTokenizerFactory,
    raw_tokenizer_factory: RawTokenizerFactory,
    simple_tokenizer_factory: SimpleTokenizerFactory,
    cang_jie_tokenizer_factory: CangJieTokenizerFactory,
    lindera_tokenizer_factory: LinderaTokenizerFactory,

    alpha_num_only_filter_factory: AlphaNumOnlyFilterFactory,
    ascii_folding_filter_factory: AsciiFoldingFilterFactory,
    lower_case_filter_factory: LowerCaseFilterFactory,
    remove_long_filter_factory: RemoveLongFilterFactory,
    stemming_filter_factory: StemmingFilterFactory,
    stop_word_filter_factory: StopWordFilterFactory,
}

impl TokenizerInitializer {
    pub fn new() -> Self {
        TokenizerInitializer {
            facet_tokenizer_factory: FacetTokenizerFactory::new(),
            ngram_tokenizer_factory: NgramTokenizerFactory::new(),
            raw_tokenizer_factory: RawTokenizerFactory::new(),
            simple_tokenizer_factory: SimpleTokenizerFactory::new(),
            cang_jie_tokenizer_factory: CangJieTokenizerFactory::new(),
            lindera_tokenizer_factory: LinderaTokenizerFactory::new(),

            alpha_num_only_filter_factory: AlphaNumOnlyFilterFactory::new(),
            ascii_folding_filter_factory: AsciiFoldingFilterFactory::new(),
            lower_case_filter_factory: LowerCaseFilterFactory::new(),
            remove_long_filter_factory: RemoveLongFilterFactory::new(),
            stemming_filter_factory: StemmingFilterFactory::new(),
            stop_word_filter_factory: StopWordFilterFactory::new(),
        }
    }

    pub fn configure(&mut self, manager: &TokenizerManager, config: &str) {
        let config_value: Value = serde_json::from_str(config).unwrap();

        let config_map = config_value.as_object().unwrap();
        for (name, tokenizer_config_value) in config_map {
            debug!("name: {}", name);

            let tokenizer_config_map = tokenizer_config_value.as_object().unwrap();

            // tokenizer
            let tokenizer_settings = tokenizer_config_map["tokenizer"].as_object().unwrap();
            debug!("tokenizer_setting: {:?}", tokenizer_settings);

            let tokenizer_name = tokenizer_settings["name"].as_str().unwrap();
            debug!("tokenizer_name: {:?}", tokenizer_name);

            let mut tokenizer_args = String::new();
            if tokenizer_settings.contains_key("args") {
                tokenizer_args = serde_json::to_string(&tokenizer_settings["args"]).unwrap();
            }
            debug!("tokenizer_args: {:?}", tokenizer_args);

            // create tokenizer
            let mut tokenizer;
            match tokenizer_name {
                "facet" => {
                    tokenizer = TextAnalyzer::from(self.facet_tokenizer_factory.clone().create());
                }
                "ngram" => {
                    tokenizer = TextAnalyzer::from(
                        self.ngram_tokenizer_factory
                            .clone()
                            .create(tokenizer_args.as_ref()),
                    );
                }
                "raw" => {
                    tokenizer = TextAnalyzer::from(self.raw_tokenizer_factory.clone().create());
                }
                "simple" => {
                    tokenizer = TextAnalyzer::from(self.simple_tokenizer_factory.clone().create());
                }
                "cang_jie" => {
                    tokenizer = TextAnalyzer::from(
                        self.cang_jie_tokenizer_factory
                            .clone()
                            .create(tokenizer_args.as_ref()),
                    );
                }
                "lindera" => {
                    tokenizer = TextAnalyzer::from(
                        self.lindera_tokenizer_factory
                            .clone()
                            .create(tokenizer_args.as_ref()),
                    );
                }
                _ => {
                    panic!("unknown tokenizer: {}", tokenizer_name);
                }
            }

            // filters
            if tokenizer_config_map.contains_key("filters") {
                let filters_config_value = tokenizer_config_map["filters"].as_array().unwrap();
                for filter_config_value in filters_config_value {
                    let filter_settings = filter_config_value.as_object().unwrap();
                    debug!("filter_settings: {:?}", filter_settings);

                    let filter_name = filter_settings["name"].as_str().unwrap();
                    debug!("filter_name: {:?}", filter_name);

                    let mut filter_args = String::new();
                    if filter_settings.contains_key("args") {
                        filter_args = serde_json::to_string(&filter_settings["args"]).unwrap();
                    }
                    debug!("filter_args: {:?}", filter_args);

                    // create filter
                    match filter_name {
                        "alpha_num_only" => {
                            tokenizer = tokenizer.filter(BoxTokenFilter::from(
                                self.alpha_num_only_filter_factory.clone().create(),
                            ));
                        }
                        "ascii_folding" => {
                            tokenizer = tokenizer.filter(BoxTokenFilter::from(
                                self.ascii_folding_filter_factory.clone().create(),
                            ));
                        }
                        "lower_case" => {
                            tokenizer = tokenizer.filter(BoxTokenFilter::from(
                                self.lower_case_filter_factory.clone().create(),
                            ));
                        }
                        "remove_long" => {
                            tokenizer = tokenizer.filter(BoxTokenFilter::from(
                                self.remove_long_filter_factory
                                    .clone()
                                    .create(filter_args.as_ref()),
                            ));
                        }
                        "stemming" => {
                            tokenizer = tokenizer.filter(BoxTokenFilter::from(
                                self.stemming_filter_factory
                                    .clone()
                                    .create(filter_args.as_ref()),
                            ));
                        }
                        "stop_word" => {
                            tokenizer = tokenizer.filter(BoxTokenFilter::from(
                                self.stop_word_filter_factory
                                    .clone()
                                    .create(filter_args.as_ref()),
                            ));
                        }
                        _ => {
                            panic!("unknown filter: {}", filter_name);
                        }
                    }
                }
            }

            manager.register(name, tokenizer)
        }

        debug!("tokenizers are initialized");
    }
}

#[cfg(test)]
mod tests {
    use tantivy::tokenizer::TokenizerManager;

    use crate::tokenizer::tokenizer_initializer::TokenizerInitializer;

    #[test]
    fn test_tokenizer() {
        let config = r#"
            {
              "lang_en": {
                "tokenizer": {
                  "name": "simple"
                },
                "filters": [
                  {
                    "name": "remove_long",
                    "args": {
                      "length_li mit": 50
                    }
                  },
                  {
                    "name": "lower_case"
                  },
                  {
                    "name": "stemming",
                    "args": {
                      "stemmer_algorithm": "english"
                    }
                  },
                  {
                    "name": "stop_word",
                    "args": {
                      "words": [
                        "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into",
                        "is", "it", "no", "not", "of", "on", "or", "such", "that", "the", "their", "then",
                        "there", "these", "they", "this", "to", "was", "will", "with"
                      ]
                    }
                  }
                ]
              }
            }
        "#;

        let manager = TokenizerManager::default();

        let mut initializer = TokenizerInitializer::new();
        initializer.configure(&manager, config);

        let tokenizer = manager.get("lang_en").unwrap();
        let mut stream = tokenizer.token_stream("I am saying HELLO WORLD!");
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "i");
            assert_eq!(token.offset_from, 0);
            assert_eq!(token.offset_to, 1);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "am");
            assert_eq!(token.offset_from, 2);
            assert_eq!(token.offset_to, 4);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "say");
            assert_eq!(token.offset_from, 5);
            assert_eq!(token.offset_to, 11);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "hello");
            assert_eq!(token.offset_from, 12);
            assert_eq!(token.offset_to, 17);
        }
        {
            let token = stream.next().unwrap();
            assert_eq!(token.text, "world");
            assert_eq!(token.offset_from, 18);
            assert_eq!(token.offset_to, 23);
        }
        assert!(stream.next().is_none());
    }
}
