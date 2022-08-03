use tantivy::{query::TermQuery, Searcher, Term};

use super::{QueryFactoryError, QueryFactoryErrorKind};

#[derive(Deserialize, Serialize, Debug)]
pub struct TermQueryArgs {
    field: String,
    term: String,
}

/// Create a term query.
pub fn create_term_query(
    args: &TermQueryArgs,
    searcher: &Searcher,
) -> Result<TermQuery, QueryFactoryError> {
    let field = searcher.schema().get_field(&args.field).ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::anyhow!("Field {} does not exist", &args.field))
    })?;

    let term = Term::from_field_text(field, &args.term);

    let segment_postings_options = searcher
        .schema()
        .get_field_entry(field)
        .field_type()
        .get_index_record_option()
        .ok_or_else(|| {
            QueryFactoryErrorKind::InvalidArgument.with_error(anyhow::anyhow!(
                "Field {} does not have index record option.",
                &args.field
            ))
        })?;

    Ok(TermQuery::new(term, segment_postings_options))
}

#[cfg(test)]
mod tests {
    use tantivy::{
        collector::{Count, TopDocs},
        doc,
        schema::{Schema, TEXT},
        Index,
    };

    use crate::search::query::term::{create_term_query, TermQueryArgs};

    #[test]
    fn test_term_query_args() {
        let opts_json_str = r#"
            {
                "field": "title",   
                "term": "test"
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deserialize the JSON into a struct.
        let args = serde_json::from_slice::<TermQueryArgs>(opts_json_bytes).unwrap();

        // check field
        assert_eq!(args.field, "title");

        // check term
        assert_eq!(args.term, "test");
    }

    fn create_test_index() -> tantivy::Result<Index> {
        let mut schema_builder = Schema::builder();
        let field = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut writer = index.writer_with_num_threads(1, 10_000_000)?;
        writer.add_document(doc!(field=>"aaa"))?;
        writer.add_document(doc!(field=>"bbb"))?;
        writer.commit()?;
        writer.add_document(doc!(field=>"ccc"))?;
        writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_term_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "field": "text",
                "term": "aaa"
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<TermQueryArgs>(opts_json_bytes).unwrap();
        let query = create_term_query(&opts, &searcher).unwrap();

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
