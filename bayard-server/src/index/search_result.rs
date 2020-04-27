use std::collections::HashMap;

use serde::Serialize;
use tantivy::schema::NamedFieldDocument;

#[derive(Serialize)]
pub struct ScoredNamedFieldDocument {
    pub fields: NamedFieldDocument,
    pub score: f32,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub count: i64,
    pub docs: Vec<ScoredNamedFieldDocument>,
    pub facet: HashMap<String, HashMap<String, u64>>,
}
