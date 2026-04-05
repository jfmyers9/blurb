#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;

use crate::data::models::{Book, DiaryEntry, Highlight, Shelf};

pub fn export_book_highlights_markdown(
    book_title: &str,
    book_author: Option<&str>,
    highlights: &[Highlight],
) -> String {
    let mut md = String::new();

    // Header
    md.push_str("# Highlights from \"");
    md.push_str(book_title);
    md.push('"');
    if let Some(author) = book_author {
        md.push_str(" by ");
        md.push_str(author);
    }
    md.push_str("\n\n");

    for h in highlights {
        if h.text.is_empty() {
            continue;
        }
        // Quote block
        md.push_str("> ");
        md.push_str(&h.text.replace('\n', "\n> "));
        md.push('\n');

        // Location line
        let mut location_parts: Vec<String> = Vec::new();
        if let (Some(start), Some(end)) = (h.location_start, h.location_end) {
            location_parts.push(format!("Location {start}-{end}"));
        } else if let Some(start) = h.location_start {
            location_parts.push(format!("Location {start}"));
        }
        if let Some(page) = h.page {
            location_parts.push(format!("Page {page}"));
        }
        if let Some(ref clipped) = h.clipped_at {
            location_parts.push(format!("Clipped on {clipped}"));
        }

        if !location_parts.is_empty() {
            md.push_str(&format!("\u{2014} {}\n", location_parts.join(" | ")));
        }

        md.push('\n');
    }

    md
}

pub fn export_all_highlights_markdown(conn: &rusqlite::Connection) -> Result<String, String> {
    let books = crate::data::commands::list_books_db(conn)?;
    let mut md = String::from("# All Highlights\n\n");

    for book in &books {
        let highlights = crate::data::commands::list_highlights_db(conn, book.id)?;
        if highlights.is_empty() {
            continue;
        }
        md.push_str(&export_book_highlights_markdown(
            &book.title,
            book.author.as_deref(),
            &highlights,
        ));
        md.push_str("---\n\n");
    }

    Ok(md)
}

pub fn export_library_json(conn: &rusqlite::Connection) -> Result<String, String> {
    let books = crate::data::commands::list_books_db(conn)?;
    let shelves = crate::data::commands::list_shelves_db(conn)?;
    let diary_entries = crate::data::commands::list_diary_entries_db(conn)?;
    let shelf_assignments = crate::data::commands::list_all_shelf_book_ids_db(conn)?;

    let mut all_highlights: Vec<Highlight> = Vec::new();
    for book in &books {
        let mut hl = crate::data::commands::list_highlights_db(conn, book.id)?;
        all_highlights.append(&mut hl);
    }

    let export = LibraryExport {
        books,
        shelves,
        diary_entries,
        shelf_assignments,
        highlights: all_highlights,
    };

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct LibraryExport {
    books: Vec<Book>,
    shelves: Vec<Shelf>,
    diary_entries: Vec<DiaryEntry>,
    shelf_assignments: Vec<(i64, i64)>,
    highlights: Vec<Highlight>,
}
