use tantivy::{query::FuzzyTermQuery, Searcher, Term};

use super::{QueryFactoryError, QueryFactoryErrorKind};

#[derive(Deserialize, Serialize)]
pub struct FuzzyTermQueryArgs {
    field: String,
    term: String,
    distance: u8,
    transposition_cost_one: bool,
    prefix: bool,
}

/// Create a fuzzy term query.
pub fn create_fuzzy_term_query(
    args: &FuzzyTermQueryArgs,
    searcher: &Searcher,
) -> Result<FuzzyTermQuery, QueryFactoryError> {
    let field = searcher.schema().get_field(&args.field).ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::anyhow!("Field {} does not exist", &args.field))
    })?;

    let term = Term::from_field_text(field, &args.term);

    if args.prefix {
        Ok(FuzzyTermQuery::new_prefix(
            term,
            args.distance,
            args.transposition_cost_one,
        ))
    } else {
        Ok(FuzzyTermQuery::new(
            term,
            args.distance,
            args.transposition_cost_one,
        ))
    }
}

#[cfg(test)]
mod tests {
    use tantivy::{
        collector::{Count, TopDocs},
        doc,
        schema::{Schema, TEXT},
        Index,
    };

    use crate::search::query::fuzzy_term::{create_fuzzy_term_query, FuzzyTermQueryArgs};

    #[test]
    fn test_fuzzy_term_query_args() {
        let opts_json_str = r#"
            {
                "field": "description",   
                "term": "rust",
                "distance": 2,
                "transposition_cost_one": true,
                "prefix": true
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();

        // Deserialize the JSON into a struct.
        let args = serde_json::from_slice::<FuzzyTermQueryArgs>(opts_json_bytes).unwrap();

        // Check field
        assert_eq!(args.field, "description");

        // Check term
        assert_eq!(args.term, "rust");

        // Check distance
        assert_eq!(args.distance, 2);

        // Check transposition_cost_one
        assert_eq!(args.transposition_cost_one, true);

        // Check prefix
        assert_eq!(args.prefix, true);
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
    fn test_fuzzy_term_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let opts_json_str = r#"
            {
                "field": "country",   
                "term": "japon",
                "distance": 1,
                "transposition_cost_one": true,
                "prefix": true
            }
        "#;
        let opts_json_bytes = opts_json_str.as_bytes();
        let opts = serde_json::from_slice::<FuzzyTermQueryArgs>(opts_json_bytes).unwrap();
        let query = create_fuzzy_term_query(&opts, &searcher).unwrap();

        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(20), Count))
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
