use crate::data::commands::{list_diary_entries_db, search_books_fts_db, search_highlights_db};
use crate::data::models::Book;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub book: Book,
    pub snippet: Option<String>,
    pub rank: f64,
}

pub fn search_library(conn: &rusqlite::Connection, query: &str) -> Vec<SearchResult> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let lower_q = trimmed.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();
    let mut seen_book_ids = std::collections::HashSet::new();

    // FTS book search (ranked)
    if let Ok(books) = search_books_fts_db(conn, trimmed) {
        for (i, book) in books.into_iter().enumerate() {
            seen_book_ids.insert(book.id);
            let snippet = book.description.as_ref().and_then(|d| {
                let lower_d = d.to_lowercase();
                lower_d.find(&lower_q).map(|pos| {
                    let start = pos.saturating_sub(40);
                    let end = (pos + lower_q.len() + 60).min(d.len());
                    let mut s = String::new();
                    if start > 0 {
                        s.push_str("...");
                    }
                    s.push_str(&d[start..end]);
                    if end < d.len() {
                        s.push_str("...");
                    }
                    s
                })
            });
            results.push(SearchResult {
                book,
                snippet,
                rank: i as f64,
            });
        }
    }

    // Highlights matching query — surface the parent book
    if let Ok(highlights) = search_highlights_db(conn, trimmed) {
        for hl in highlights.iter().take(10) {
            if seen_book_ids.contains(&hl.book_id) {
                continue;
            }
            seen_book_ids.insert(hl.book_id);
            let snippet_text = hl.text[..hl.text.len().min(120)].to_string();
            results.push(SearchResult {
                book: Book {
                    id: hl.book_id,
                    title: hl.book_title.clone(),
                    author: hl.book_author.clone(),
                    isbn: None,
                    asin: None,
                    cover_url: None,
                    description: None,
                    publisher: None,
                    published_date: None,
                    page_count: None,
                    created_at: String::new(),
                    updated_at: String::new(),
                    rating: None,
                    status: None,
                    started_at: None,
                    finished_at: None,
                },
                snippet: Some(snippet_text),
                rank: 100.0 + results.len() as f64,
            });
        }
    }

    // Diary entries matching query — surface the parent book
    if let Ok(entries) = list_diary_entries_db(conn) {
        for entry in entries {
            if seen_book_ids.contains(&entry.book_id) {
                continue;
            }
            let body_matches = entry
                .body
                .as_ref()
                .is_some_and(|b| b.to_lowercase().contains(&lower_q));
            let title_matches = entry.book_title.to_lowercase().contains(&lower_q);
            if !body_matches && !title_matches {
                continue;
            }
            seen_book_ids.insert(entry.book_id);
            let snippet = entry.body.as_ref().map(|b| {
                let preview = &b[..b.len().min(100)];
                preview.to_string()
            });
            results.push(SearchResult {
                book: Book {
                    id: entry.book_id,
                    title: entry.book_title.clone(),
                    author: entry.book_author.clone(),
                    isbn: None,
                    asin: None,
                    cover_url: entry.book_cover_url.clone(),
                    description: None,
                    publisher: None,
                    published_date: None,
                    page_count: None,
                    created_at: String::new(),
                    updated_at: String::new(),
                    rating: entry.rating,
                    status: None,
                    started_at: None,
                    finished_at: None,
                },
                snippet,
                rank: 200.0 + results.len() as f64,
            });
        }
    }

    results
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
