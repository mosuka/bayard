use tantivy::query::AllQuery;

/// Create an all query.
pub fn create_all_query() -> AllQuery {
    AllQuery {}
}

#[cfg(test)]
mod tests {
    use tantivy::{
        collector::{Count, TopDocs},
        doc,
        schema::{Schema, TEXT},
        Index,
    };

    use crate::search::query::all::create_all_query;

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
    fn test_all_query() {
        let index = create_test_index().unwrap();
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();

        let query = create_all_query();
        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(10), Count))
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
