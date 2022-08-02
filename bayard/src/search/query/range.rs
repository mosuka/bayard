use std::ops::Range;

use tantivy::{query::RangeQuery, schema::Type, Searcher};

use super::{QueryFactoryError, QueryFactoryErrorKind};

pub type RangeQueryArgs = serde_json::Value;

/// Create a range query.
pub fn create_range_query(
    args: &RangeQueryArgs,
    searcher: &Searcher,
) -> Result<RangeQuery, QueryFactoryError> {
    let args_map = args.as_object().ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::format_err!("Content is not an object"))
    })?;

    let field_str = args_map
        .get("field")
        .ok_or_else(|| {
            QueryFactoryErrorKind::InvalidArgument
                .with_error(anyhow::format_err!("field does not exist in arguments"))
        })?
        .as_str()
        .ok_or_else(|| {
            QueryFactoryErrorKind::InvalidArgument
                .with_error(anyhow::format_err!("field is not a string type"))
        })?;

    let start_value = args_map.get("start").ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::format_err!("start does not exist in arguments"))
    })?;

    let end_value = args_map.get("end").ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::format_err!("end does not exist in arguments"))
    })?;

    let field = searcher.schema().get_field(field_str).ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::anyhow!("Field {} does not exist", field_str))
    })?;

    let field_type = searcher
        .schema()
        .get_field_entry(field)
        .field_type()
        .value_type();

    match field_type {
        Type::I64 => {
            let start = start_value.as_i64().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("start is not an i64 type"))
            })?;
            let end = end_value.as_i64().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("end is not an i64 type"))
            })?;

            Ok(RangeQuery::new_i64(field, Range { start, end }))
        }
        Type::F64 => {
            let start = start_value.as_f64().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("start is not a f64 type"))
            })?;
            let end = end_value.as_f64().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("end is not a f64 type"))
            })?;

            Ok(RangeQuery::new_f64(field, Range { start, end }))
        }
        Type::U64 => {
            let start = start_value.as_u64().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("start is not a u64 type"))
            })?;
            let end = end_value.as_u64().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("end is not a u64 type"))
            })?;

            Ok(RangeQuery::new_u64(field, Range { start, end }))
        }
        Type::Str => {
            let start = start_value.as_str().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("start is not a string type"))
            })?;
            let end = end_value.as_str().ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::format_err!("end is not a string type"))
            })?;

            Ok(RangeQuery::new_str(field, Range { start, end }))
        }
        _ => Err(
            QueryFactoryErrorKind::InvalidArgument.with_error(anyhow::format_err!(
                "Unsupported field type: {:?}",
                field_type
            )),
        ),
    }
}

#[cfg(test)]
mod tests {
    use tantivy::{
        collector::{Count, TopDocs},
        doc,
        schema::{Schema, INDEXED},
        Index,
    };

    use crate::search::query::range::{create_range_query, RangeQueryArgs};

    #[test]
    fn test_range_query_args() {
        let opts_json_str = r#"
            {
                "field": "popularity",   
                "start": 10000,
                "end": 20000
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deserialize the JSON into a struct.
        let args = serde_json::from_slice::<RangeQueryArgs>(opts_json_bytes).unwrap();

        // Create args map.
        let args_map = args.as_object().unwrap();

        // check field
        let field_str = args_map.get("field").unwrap().as_str().unwrap();
        assert_eq!(field_str, "popularity");
    }

    fn create_test_index() -> tantivy::Result<Index> {
        let mut schema_builder = Schema::builder();
        let year_field = schema_builder.add_u64_field("year", INDEXED);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut writer = index.writer_with_num_threads(1, 10_000_000)?;
        for year in 1950u64..2017u64 {
            writer.add_document(doc!(year_field => year))?;
        }
        writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_range_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "field": "year",   
                "start": 1960,
                "end": 1970
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<RangeQueryArgs>(opts_json_bytes).unwrap();
        let query = create_range_query(&opts, &searcher).unwrap();

        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(20), Count))
            .unwrap();

        let mut docs = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address).unwrap();
            docs.push(doc);
        }

        assert_eq!(docs.len(), 10);

        assert_eq!(count, 10);
    }
}
