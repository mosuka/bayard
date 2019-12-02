use serde::Serialize;
use tantivy::schema::NamedFieldDocument;

#[derive(Serialize)]
pub struct ScoredNamedFieldDocument {
    pub fields: NamedFieldDocument,
    pub score: f32,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub count: usize,
    pub docs: Vec<ScoredNamedFieldDocument>,
}
