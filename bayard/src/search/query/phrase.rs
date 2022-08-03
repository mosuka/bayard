use tantivy::{query::PhraseQuery, Searcher, Term};

use super::{QueryFactoryError, QueryFactoryErrorKind};

#[derive(Deserialize, Serialize)]
pub struct PhraseQueryArgs {
    phrase_terms: Vec<String>,
    field: String,
    slop: u32,
}

/// Create a phrase query.
pub fn create_phrase_query(
    args: &PhraseQueryArgs,
    searcher: &Searcher,
) -> Result<PhraseQuery, QueryFactoryError> {
    let field = searcher.schema().get_field(&args.field).ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::anyhow!("Field {} does not exist", &args.field))
    })?;

    let phrase_terms: Vec<Term> = args
        .phrase_terms
        .iter()
        .map(|phrase_term| Term::from_field_text(field, phrase_term))
        .collect();

    let mut query = PhraseQuery::new(phrase_terms);

    query.set_slop(args.slop);

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

    use crate::search::query::phrase::{create_phrase_query, PhraseQueryArgs};

    #[test]
    fn test_phrase_query_args() {
        let opts_json_str = r#"
            {
                "phrase_terms": ["distributed", "search", "engine"],
                "field": "description",
                "slop": 1
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deserialize the JSON into a struct.
        let args = serde_json::from_slice::<PhraseQueryArgs>(opts_json_bytes).unwrap();

        // Check phrase terms
        assert_eq!(args.phrase_terms, vec!["distributed", "search", "engine"]);

        // Check field
        assert_eq!(args.field, "description");

        // Check term
        assert_eq!(args.slop, 1);
    }

    fn create_test_index() -> tantivy::Result<Index> {
        let mut schema_builder = Schema::builder();
        let text = schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        let mut writer = index.writer_with_num_threads(1, 10_000_000)?;
        writer.add_document(doc!(text=>"b b b d c g c"))?;
        writer.add_document(doc!(text=>"a b b d c g c"))?;
        writer.add_document(doc!(text=>"a b a b c"))?;
        writer.add_document(doc!(text=>"c a b a d ga a"))?;
        writer.add_document(doc!(text=>"a b c"))?;
        writer.commit()?;
        Ok(index)
    }

    #[test]
    fn test_phrase_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "phrase_terms": ["a", "b", "c"],
                "field": "text",
                "slop": 0
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<PhraseQueryArgs>(opts_json_bytes).unwrap();
        let query = create_phrase_query(&opts, &searcher).unwrap();

        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(20), Count))
            .unwrap();

        let mut docs = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address).unwrap();
            docs.push(doc);
        }

        assert_eq!(docs.len(), 2);

        assert_eq!(count, 2);
    }
}
