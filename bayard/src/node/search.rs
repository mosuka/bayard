use std::cmp::Reverse;

use tantivy::{
    collector::{Count, MultiCollector, TopDocs},
    fastfield::FastFieldReader,
    query::Query,
    DateTime, DocAddress, DocId, Document as TDocument, LeasedItem, Searcher, SegmentReader,
};

use crate::{
    index::{DOC_ID_FIELD_NAME, DOC_TIMESTAMP_FIELD_NAME},
    proto::index::{CollectionKind, Document, Sort},
};

use super::{NodeError, NodeErrorKind};

pub fn search_order_by_score_desc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector
                .add_collector(TopDocs::with_limit(hits as usize).and_offset(offset as usize)),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(f32, DocAddress)> = top_docs;
    for (score, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score,
            timestamp,
            sort_value: 0.0,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_i64_asc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .custom_score(move |segment_reader: &SegmentReader| {
                        let reader = segment_reader
                            .fast_fields()
                            .i64(sort_field)
                            .expect("field exists");

                        move |doc: DocId| {
                            let value: i64 = reader.get(doc);
                            Reverse(value)
                        }
                    }),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(Reverse<i64>, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value: sort_value.0 as f64,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_i64_desc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .order_by_fast_field(sort_field),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(i64, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value: sort_value as f64,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_f64_asc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .custom_score(move |segment_reader: &SegmentReader| {
                        let reader = segment_reader
                            .fast_fields()
                            .f64(sort_field)
                            .expect("field exists");

                        move |doc: DocId| {
                            let value: f64 = reader.get(doc);
                            Reverse(value)
                        }
                    }),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(Reverse<f64>, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value: sort_value.0,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_f64_desc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .order_by_fast_field(sort_field),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(f64, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_u64_asc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .custom_score(move |segment_reader: &SegmentReader| {
                        let reader = segment_reader
                            .fast_fields()
                            .u64(sort_field)
                            .expect("field exists");

                        move |doc: DocId| {
                            let value: u64 = reader.get(doc);
                            Reverse(value)
                        }
                    }),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(Reverse<u64>, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value: sort_value.0 as f64,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_u64_desc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .order_by_fast_field(sort_field),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(u64, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value: sort_value as f64,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_date_asc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .custom_score(move |segment_reader: &SegmentReader| {
                        let reader = segment_reader
                            .fast_fields()
                            .date(sort_field)
                            .expect("field exists");

                        move |doc: DocId| {
                            let value: DateTime = reader.get(doc);
                            Reverse(value)
                        }
                    }),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(Reverse<DateTime>, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value: sort_value.0.into_utc().unix_timestamp() as f64,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}

pub fn search_order_by_date_desc(
    searcher: &LeasedItem<Searcher>,
    query: Box<dyn Query>,
    hits: i32,
    offset: i32,
    sort: Sort,
    fields: Vec<String>,
    kind: CollectionKind,
) -> Result<(i64, Vec<Document>), NodeError> {
    // Get schema.
    let schema = searcher.schema();

    // Sort field.
    let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "Sort field {:?} does not exist.",
            sort.field
        ))
    })?;

    // Create multi collector.
    let mut multi_collector = MultiCollector::new();

    // Create count collector.
    let count_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::Count => {
            Some(multi_collector.add_collector(Count))
        }
        _ => None,
    };

    // Create top docs collector.
    let top_docs_handle = match kind {
        CollectionKind::CountAndTopDocs | CollectionKind::TopDocs => Some(
            multi_collector.add_collector(
                TopDocs::with_limit(hits as usize)
                    .and_offset(offset as usize)
                    .order_by_fast_field(sort_field),
            ),
        ),
        _ => None,
    };

    // Search index.
    let mut multi_fruit = searcher
        .search(&query, &multi_collector)
        .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

    // Get total hits count.
    let total_hits = if let Some(handle) = count_handle {
        handle.extract(&mut multi_fruit) as i64
    } else {
        // If no count is requested, return -1.
        -1
    };

    // Get top docs.
    let top_docs = if let Some(handle) = top_docs_handle {
        handle.extract(&mut multi_fruit)
    } else {
        // If no top docs are requested, return None.
        Vec::new()
    };

    // Get document ID field.
    let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_ID_FIELD_NAME
        ))
    })?;

    // Get document timestamp field.
    let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
        NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
            "{:?} field does not exist.",
            DOC_TIMESTAMP_FIELD_NAME
        ))
    })?;

    // Create documents.
    let mut documents = Vec::new();
    let top_docs: Vec<(DateTime, DocAddress)> = top_docs;
    for (sort_value, doc_addr) in top_docs {
        let doc = searcher
            .doc(doc_addr)
            .map_err(|error| NodeErrorKind::IndexSearchFailure.with_error(error))?;

        let id = doc
            .get_first(doc_id_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure
                    .with_error(anyhow::anyhow!("{:?} field does not exist.", doc_id_field))
            })?
            .as_text()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .to_string();

        let timestamp = doc
            .get_first(doc_timestamp_field)
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field does not exist.",
                    doc_timestamp_field
                ))
            })?
            .as_date()
            .ok_or_else(|| {
                NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                    "{:?} field is not text type.",
                    doc_id_field
                ))
            })?
            .into_utc()
            .unix_timestamp();

        let mut new_doc = TDocument::new();
        for field in fields.iter() {
            for (doc_field, doc_field_values) in doc.get_sorted_field_values() {
                if field == schema.get_field_name(doc_field)
                    && (field != DOC_ID_FIELD_NAME || field != DOC_TIMESTAMP_FIELD_NAME)
                {
                    for doc_field_value in doc_field_values.into_iter().cloned() {
                        new_doc.add_field_value(doc_field, doc_field_value);
                    }
                }
            }
        }

        let doc_json = schema.to_json(&new_doc);

        let doc = Document {
            id,
            score: 0.0,
            timestamp,
            sort_value: sort_value.into_utc().unix_timestamp() as f64,
            fields: doc_json.as_bytes().to_vec(),
        };

        documents.push(doc);
    }

    Ok((total_hits, documents))
}
