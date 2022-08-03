use tantivy::{
    query::{Query, QueryParser},
    schema::Field,
    Searcher,
};

use super::{QueryFactoryError, QueryFactoryErrorKind};

#[derive(Deserialize, Serialize)]
pub struct QueryStringQueryArgs {
    query: String,
    default_search_fields: Vec<String>,
}

/// Create a query string query.
pub fn create_query_string_query(
    args: &QueryStringQueryArgs,
    searcher: &Searcher,
) -> Result<Box<dyn Query>, QueryFactoryError> {
    let default_fields: Vec<Field> = if args.default_search_fields.is_empty() {
        return Err(QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::anyhow!("No default search fields provided.")));
    } else {
        let mut fields = Vec::new();
        for field_name in args.default_search_fields.iter() {
            let field = searcher.schema().get_field(field_name).ok_or_else(|| {
                QueryFactoryErrorKind::InvalidArgument
                    .with_error(anyhow::anyhow!("Field {} does not exist", field_name))
            })?;
            fields.push(field);
        }
        fields
    };

    let query_parser = QueryParser::for_index(searcher.index(), default_fields);

    query_parser
        .parse_query(&args.query)
        .map_err(|error| QueryFactoryErrorKind::InvalidArgument.with_error(error))
}

#[cfg(test)]
mod tests {
    use tantivy::{
        collector::{Count, TopDocs},
        doc,
        schema::{Schema, TEXT},
        Index,
    };

    use crate::search::query::query_string::{create_query_string_query, QueryStringQueryArgs};

    #[test]
    fn test_query_string_query_args() {
        let opts_json_str = r#"
            {
                "query": "search engine",   
                "default_search_fields": ["name", "description"]
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deserialize the JSON into a struct.
        let args = serde_json::from_slice::<QueryStringQueryArgs>(opts_json_bytes).unwrap();

        // Check query
        assert_eq!(args.query, "search engine");

        // Check default search fields
        assert_eq!(
            args.default_search_fields,
            vec!["name".to_string(), "description".to_string()]
        );
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
    fn test_create_query_string_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "query": "\"a b\"",
                "default_search_fields": ["text"]
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<QueryStringQueryArgs>(opts_json_bytes).unwrap();
        let query = create_query_string_query(&opts, &searcher).unwrap();

        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(10), Count))
            .unwrap();

        let mut docs = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address).unwrap();
            docs.push(doc);
        }

        assert_eq!(docs.len(), 1);

        assert_eq!(count, 1);
    }
}
