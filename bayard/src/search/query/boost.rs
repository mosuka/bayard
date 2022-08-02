use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Unexpected, Visitor};
use tantivy::{query::BoostQuery, Searcher};

use crate::proto::index::{query::Kind as ProtoQueryKind, Query as ProtoQuery};

use super::{create_query, QueryFactoryError};

#[derive(Serialize, Debug)]
pub struct BoostQueryArgs {
    query: ProtoQuery,
    boost: f32,
}

impl BoostQueryArgs {
    pub fn new(query: ProtoQuery, boost: f32) -> Self {
        Self { query, boost }
    }
}

impl<'de> Deserialize<'de> for BoostQueryArgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["query", "score"];

        enum Field {
            Query,
            Boost,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`query` or `boost`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "query" => Ok(Field::Query),
                            "boost" => Ok(Field::Boost),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct BoostQueryArgsVisitor;

        impl<'de> Visitor<'de> for BoostQueryArgsVisitor {
            type Value = BoostQueryArgs;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BoostQueryArgs")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<BoostQueryArgs, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let query = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let boost = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(BoostQueryArgs::new(query, boost))
            }

            fn visit_map<V>(self, mut map: V) -> Result<BoostQueryArgs, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut query = None;
                let mut boost = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Query => {
                            if query.is_some() {
                                return Err(de::Error::duplicate_field("query"));
                            }

                            let tmp_query = map.next_value::<serde_json::Value>()?;

                            let query_kind_str = tmp_query["kind"].as_str().ok_or_else(|| {
                                de::Error::invalid_type(Unexpected::Other("unexpected type"), &self)
                            })?;

                            let query_kind =
                                ProtoQueryKind::from_str(query_kind_str).ok_or_else(|| {
                                    de::Error::custom(format!(
                                        "failed to convert query type {:?} from string",
                                        query_kind_str
                                    ))
                                })?;

                            let query_opts = match tmp_query["options"].as_object() {
                                Some(opts) => serde_json::to_vec(&opts)
                                    .map_err(|e| de::Error::custom(format!("{}", e)))?,
                                None => Vec::new(),
                            };

                            query = Some(ProtoQuery {
                                kind: query_kind as i32,
                                options: query_opts,
                            });
                        }
                        Field::Boost => {
                            if boost.is_some() {
                                return Err(de::Error::duplicate_field("boost"));
                            }
                            boost = Some(map.next_value()?);
                        }
                    }
                }
                let query = query.ok_or_else(|| de::Error::missing_field("query"))?;
                let boost = boost.ok_or_else(|| de::Error::missing_field("boost"))?;
                Ok(BoostQueryArgs::new(query, boost))
            }
        }

        deserializer.deserialize_struct("BoostQueryArgs", FIELDS, BoostQueryArgsVisitor)
    }
}

/// Create a boost query.
pub fn create_boost_query(
    args: &BoostQueryArgs,
    searcher: &Searcher,
) -> Result<BoostQuery, QueryFactoryError> {
    let query = create_query(&args.query, searcher)?;

    Ok(BoostQuery::new(query, args.boost))
}

#[cfg(test)]
mod tests {
    use tantivy::query::Query;
    use tantivy::schema::Schema;
    use tantivy::{DocAddress, Document, Index};

    use crate::search::query::boost::{create_boost_query, BoostQueryArgs};

    #[test]
    fn test_boost_query_args() {
        let opts_json_str = r#"
            {
                "query": {
                    "kind": "term",
                    "options": {
                        "term": "rust",
                        "field": "description"
                    }
                },
                "boost": 2.0
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deserialize the JSON into a struct.
        let args = serde_json::from_slice::<BoostQueryArgs>(opts_json_bytes).unwrap();

        // check boost
        assert_eq!(args.boost, 2.0);
    }

    fn create_test_index() -> tantivy::Result<Index> {
        let schema = Schema::builder().build();
        let index = Index::create_in_ram(schema);
        let mut writer = index.writer_with_num_threads(1, 10_000_000)?;
        writer.add_document(Document::new())?;
        writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_boost_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "query": {
                    "kind": "all"
                },
                "boost": 2.0
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<BoostQueryArgs>(opts_json_bytes).unwrap();
        let query = create_boost_query(&opts, &searcher).unwrap();

        let explanation = query.explain(&searcher, DocAddress::new(0, 0u32)).unwrap();
        assert_eq!(
            explanation.to_pretty_json(),
            "{\n  \"value\": 2.0,\n  \"description\": \"Boost x2 of ...\",\n  \"details\": [\n    {\n      \"value\": 1.0,\n      \"description\": \"AllQuery\",\n      \"context\": []\n    }\n  ],\n  \"context\": []\n}"
        );
    }
}
