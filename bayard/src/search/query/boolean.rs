use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Unexpected, Visitor};
use tantivy::{
    query::{BooleanQuery, Occur, Query},
    Searcher,
};

use crate::proto::index::{query::Kind as ProtoQueryKind, Query as ProtoQuery};

use super::{create_query, QueryFactoryError};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum Occurrence {
    #[serde(rename = "must")]
    Must,
    #[serde(rename = "must_not")]
    MustNot,
    #[serde(rename = "should")]
    Should,
}

#[derive(Serialize, Debug)]
pub struct SubQuery {
    pub occurrence: Occurrence,
    pub query: ProtoQuery,
}

impl SubQuery {
    pub fn new(occurrence: Occurrence, query: ProtoQuery) -> Self {
        Self { occurrence, query }
    }
}

impl<'de> Deserialize<'de> for SubQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["occurrence", "query"];

        enum Field {
            Occurrence,
            Query,
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
                        formatter.write_str("`occurrence` or `query`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "occurrence" => Ok(Field::Occurrence),
                            "query" => Ok(Field::Query),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct SubQueryVisitor;

        impl<'de> Visitor<'de> for SubQueryVisitor {
            type Value = SubQuery;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SubQuery")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<SubQuery, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let occurrence = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let query = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SubQuery::new(occurrence, query))
            }

            fn visit_map<V>(self, mut map: V) -> Result<SubQuery, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut occurrence = None;
                let mut query = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Occurrence => {
                            if occurrence.is_some() {
                                return Err(de::Error::duplicate_field("occurrence"));
                            }
                            occurrence = Some(map.next_value()?);
                        }
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
                    }
                }
                let occurrence =
                    occurrence.ok_or_else(|| de::Error::missing_field("occurrence"))?;
                let query = query.ok_or_else(|| de::Error::missing_field("query"))?;
                Ok(SubQuery::new(occurrence, query))
            }
        }

        deserializer.deserialize_struct("SubQuery", FIELDS, SubQueryVisitor)
    }
}

#[derive(Serialize, Debug)]
pub struct BooleanQueryArgs {
    subqueries: Vec<SubQuery>,
}

impl BooleanQueryArgs {
    pub fn new(subqueries: Vec<SubQuery>) -> Self {
        Self { subqueries }
    }
}

impl<'de> Deserialize<'de> for BooleanQueryArgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["subqueries"];

        enum Field {
            SubQueries,
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
                        formatter.write_str("`subqueries`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "subqueries" => Ok(Field::SubQueries),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct BooleanQueryArgsVisitor;

        impl<'de> Visitor<'de> for BooleanQueryArgsVisitor {
            type Value = BooleanQueryArgs;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BooleanQueryArgs")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<BooleanQueryArgs, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let subqueries = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(BooleanQueryArgs::new(subqueries))
            }

            fn visit_map<V>(self, mut map: V) -> Result<BooleanQueryArgs, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut subqueries = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::SubQueries => {
                            if subqueries.is_some() {
                                return Err(de::Error::duplicate_field("subqueries"));
                            }

                            let tmp_subqueries_json = map.next_value::<serde_json::Value>()?;

                            let tmp_subquery_array =
                                tmp_subqueries_json.as_array().ok_or_else(|| {
                                    de::Error::invalid_type(
                                        Unexpected::Other("unexpected type"),
                                        &self,
                                    )
                                })?;

                            let mut tmp_subqueries = Vec::new();
                            for tmp_subquery in tmp_subquery_array.iter() {
                                let subquery =
                                    serde_json::from_value::<SubQuery>(tmp_subquery.clone())
                                        .map_err(|e| de::Error::custom(format!("{}", e)))?;
                                tmp_subqueries.push(subquery);
                            }

                            subqueries = Some(tmp_subqueries);
                        }
                    }
                }

                let subqueries =
                    subqueries.ok_or_else(|| de::Error::missing_field("subqueries"))?;
                Ok(BooleanQueryArgs::new(subqueries))
            }
        }

        deserializer.deserialize_struct("BooleanQueryArgs", FIELDS, BooleanQueryArgsVisitor)
    }
}

/// Create a boolean query.
pub fn create_boolean_query(
    args: &BooleanQueryArgs,
    searcher: &Searcher,
) -> Result<BooleanQuery, QueryFactoryError> {
    let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
    for subquery in args.subqueries.iter() {
        let occurrence = match subquery.occurrence {
            Occurrence::Must => Occur::Must,
            Occurrence::MustNot => Occur::MustNot,
            Occurrence::Should => Occur::Should,
        };

        let query = create_query(&subquery.query, searcher)?;

        subqueries.push((occurrence, query));
    }

    Ok(BooleanQuery::new(subqueries))
}

#[cfg(test)]
mod tests {
    use tantivy::{
        collector::{Count, TopDocs},
        doc,
        schema::{Schema, TEXT},
        Index,
    };

    use crate::{
        proto::index::query::Kind as ProtoQueryKind,
        search::query::boolean::{create_boolean_query, BooleanQueryArgs, Occurrence, SubQuery},
    };

    #[test]
    fn test_subquery() {
        let opts_json_str = r#"
            {
                "occurrence": "must",
                "query": {
                    "kind": "term",
                    "options": {
                        "term": "hello",
                        "field": "description"
                    }
                }
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deseerialize the JSON into a struct.
        let args = serde_json::from_slice::<SubQuery>(opts_json_bytes).unwrap();

        // Check occurrence.
        assert_eq!(args.occurrence, Occurrence::Must);

        // Check query kind.
        assert_eq!(args.query.kind, ProtoQueryKind::Term as i32);

        // Check query options.
        let expected_opts_json_str = r#"
            {
                "term": "hello",
                "field": "description"
            }
        "#;
        let expected_opts_json =
            serde_json::from_str::<serde_json::Value>(expected_opts_json_str).unwrap();
        let expected_opts_bytes = serde_json::to_vec(&expected_opts_json).unwrap();
        assert_eq!(args.query.options, expected_opts_bytes);
    }

    #[test]
    fn test_boolean_query_args() {
        let opts_json_str = r#"
            {
                "subqueries": [
                    {
                        "occurrence": "must",
                        "query": {
                            "kind": "term",
                            "options": {
                                "term": "hello",
                                "field": "description"
                            }
                        }
                    },
                    {
                        "occurrence": "must_not",
                        "query": {
                            "kind": "term",
                            "options": {
                                "term": "bye",
                                "field": "description"
                            }
                        }
                    },
                    {
                        "occurrence": "should",
                        "query": {
                            "kind": "term",
                            "options": {
                                "term": "hi",
                                "field": "description"
                            }
                        }
                    }
                ]
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deseerialize the JSON into a struct.
        let args = serde_json::from_slice::<BooleanQueryArgs>(opts_json_bytes).unwrap();

        // Check subqueries length.
        assert_eq!(args.subqueries.len(), 3);

        // Check first subquery.
        let subquery1 = &args.subqueries[0];
        assert_eq!(subquery1.occurrence, Occurrence::Must);
        assert_eq!(subquery1.query.kind, ProtoQueryKind::Term as i32);

        // Check second subquery.
        let subquery2 = &args.subqueries[1];
        assert_eq!(subquery2.occurrence, Occurrence::MustNot);
        assert_eq!(subquery2.query.kind, ProtoQueryKind::Term as i32);

        // Check third subquery.
        let subquery3 = &args.subqueries[2];
        assert_eq!(subquery3.occurrence, Occurrence::Should);
        assert_eq!(subquery3.query.kind, ProtoQueryKind::Term as i32);
    }

    fn create_test_index() -> tantivy::Result<Index> {
        let mut schema_builder = Schema::builder();
        let text = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut writer = index.writer_with_num_threads(1, 10_000_000)?;
        writer.add_document(doc!(text=>"b c"))?;
        writer.add_document(doc!(text=>"a c"))?;
        writer.add_document(doc!(text=>"a b"))?;
        writer.add_document(doc!(text=>"a d"))?;
        writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_boolean_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "subqueries": [
                    {
                        "occurrence": "should",
                        "query": {
                            "kind": "term",
                            "options": {
                                "term": "a",
                                "field": "text"
                            }
                        }
                    },
                    {
                        "occurrence": "should",
                        "query": {
                            "kind": "term",
                            "options": {
                                "term": "d",
                                "field": "text"
                            }
                        }
                    }
                ]
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<BooleanQueryArgs>(opts_json_bytes).unwrap();
        let query = create_boolean_query(&opts, &searcher).unwrap();

        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(20), Count))
            .unwrap();

        let mut docs = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address).unwrap();
            docs.push(doc);
        }

        assert_eq!(docs.len(), 3);

        assert_eq!(count, 3);
    }
}
