use dioxus::prelude::*;
use tracing::{error, info, warn};

use crate::data::commands::{enrich_book_db, get_book_db};
use crate::services::metadata;
use crate::DatabaseHandle;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImportSource {
    Goodreads,
    Kindle,
}

impl ImportSource {
    pub fn label(self) -> &'static str {
        match self {
            ImportSource::Goodreads => "Goodreads import",
            ImportSource::Kindle => "Kindle import",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnrichmentStatus {
    Idle,
    Running {
        current: usize,
        total: usize,
        current_title: String,
        source: ImportSource,
    },
    Done {
        succeeded: usize,
        failed: usize,
    },
}

#[derive(Clone, Copy)]
pub struct EnrichmentState {
    pub status: Signal<EnrichmentStatus>,
    pub cancelled: Signal<bool>,
}

impl EnrichmentState {
    pub fn new() -> Self {
        Self {
            status: Signal::new(EnrichmentStatus::Idle),
            cancelled: Signal::new(false),
        }
    }
}

pub async fn run_enrichment(
    db: DatabaseHandle,
    mut state: EnrichmentState,
    book_ids: Vec<i64>,
    source: ImportSource,
) {
    if matches!(*state.status.read(), EnrichmentStatus::Running { .. }) {
        warn!("enrichment already running, skipping");
        return;
    }

    state.cancelled.set(false);
    let books: Vec<_> = {
        let conn = db.conn.lock().unwrap();
        book_ids
            .iter()
            .filter_map(|&id| get_book_db(&conn, id).ok())
            .filter(|b| b.author.is_none() || b.cover_url.is_none())
            .collect()
    };

    if books.is_empty() {
        return;
    }

    let total = books.len();
    let mut succeeded = 0usize;
    let mut failed = 0usize;

    for (i, book) in books.into_iter().enumerate() {
        if *state.cancelled.read() {
            break;
        }

        state.status.set(EnrichmentStatus::Running {
            current: i + 1,
            total,
            current_title: book.title.clone(),
            source,
        });

        let meta_result = match book.isbn.as_deref().filter(|s| !s.is_empty()) {
            Some(isbn) => match metadata::lookup(isbn).await {
                Ok(meta) if meta.title.is_some() => {
                    info!(book_id = book.id, isbn, "enriching via ISBN lookup");
                    Ok(meta)
                }
                _ => {
                    info!(
                        book_id = book.id,
                        isbn, "ISBN lookup failed, falling back to title search"
                    );
                    metadata::search_by_title(&book.title, book.author.as_deref()).await
                }
            },
            None => {
                info!(book_id = book.id, "enriching via title search (no ISBN)");
                metadata::search_by_title(&book.title, book.author.as_deref()).await
            }
        };

        match meta_result {
            Ok(meta) => {
                let conn = db.conn.lock().unwrap();
                if let Err(e) = enrich_book_db(&conn, book.id, &meta) {
                    error!("enrich book {}: {e}", book.id);
                    failed += 1;
                } else {
                    info!(
                        book_id = book.id,
                        title = book.title,
                        "enriched ({}/{})",
                        i + 1,
                        total
                    );
                    succeeded += 1;
                }
            }
            Err(e) => {
                error!("metadata fetch for book {}: {e}", book.id);
                failed += 1;
            }
        }
    }

    state
        .status
        .set(EnrichmentStatus::Done { succeeded, failed });
}
