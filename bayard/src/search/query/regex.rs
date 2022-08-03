use tantivy::{query::RegexQuery, Searcher};

use super::{QueryFactoryError, QueryFactoryErrorKind};

#[derive(Deserialize, Serialize)]
pub struct RegexQueryArgs {
    field: String,
    regex: String,
}

/// Create a regex query.
pub fn create_regex_query(
    args: &RegexQueryArgs,
    searcher: &Searcher,
) -> Result<RegexQuery, QueryFactoryError> {
    let field = searcher.schema().get_field(&args.field).ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::anyhow!("Field {} does not exist", &args.field))
    })?;

    let query = RegexQuery::from_pattern(&args.regex, field)
        .map_err(|error| QueryFactoryErrorKind::InvalidArgument.with_error(error))?;

    Ok(query)
}

#[cfg(test)]
mod tests {
    use tantivy::{
        collector::{Count, TopDocs},
        doc,
        schema::{Schema, TEXT},
        Index,
    };

    use crate::search::query::regex::{create_regex_query, RegexQueryArgs};

    #[test]
    fn test_regex_query_args() {
        let opts_json_str = r#"
            {
                "field": "description",   
                "regex": "ru.+t"
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deserialize the JSON into a struct.
        let args = serde_json::from_slice::<RegexQueryArgs>(opts_json_bytes).unwrap();

        // check field
        assert_eq!(args.field, "description");

        // check term
        assert_eq!(args.regex, "ru.+t");
    }

    fn create_test_index() -> tantivy::Result<Index> {
        let mut schema_builder = Schema::builder();
        let country_field = schema_builder.add_text_field("country", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut writer = index.writer_with_num_threads(1, 10_000_000)?;
        writer.add_document(doc!(
            country_field => "japan",
        ))?;
        writer.add_document(doc!(
            country_field => "korea",
        ))?;
        writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_regex_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "field": "country",   
                "regex": "jap[ao]n"
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<RegexQueryArgs>(opts_json_bytes).unwrap();
        let query = create_regex_query(&opts, &searcher).unwrap();

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
