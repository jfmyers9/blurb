use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rusqlite::OptionalExtension;

use crate::data::models::{
    Book, BookNote, Collection, DiaryEntry, Highlight, HighlightSearchResult, ReadingGoal,
    ReadingGoalProgress, ReadingStats, Shelf,
};
use crate::services::goodreads::ParsedGoodreadsBook;
use crate::services::kindle::KindleBook;
use crate::services::metadata::BookMetadata;

const BOOK_SELECT: &str = "SELECT b.id, b.title, b.author, b.isbn, b.asin, \
    b.cover_url, b.description, b.publisher, b.published_date, \
    b.page_count, b.created_at, b.updated_at, \
    r.score, rs.status, rs.started_at, rs.finished_at \
    FROM books b \
    LEFT JOIN ratings r ON r.book_id = b.id \
    LEFT JOIN reading_status rs ON rs.book_id = b.id";

fn row_to_book(row: &rusqlite::Row) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get(0)?,
        title: row.get(1)?,
        author: row.get(2)?,
        isbn: row.get(3)?,
        asin: row.get(4)?,
        cover_url: row.get(5)?,
        description: row.get(6)?,
        publisher: row.get(7)?,
        published_date: row.get(8)?,
        page_count: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        rating: row.get(12)?,
        status: row.get(13)?,
        started_at: row.get(14)?,
        finished_at: row.get(15)?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn add_book_db(
    conn: &rusqlite::Connection,
    title: &str,
    author: Option<&str>,
    isbn: Option<&str>,
    asin: Option<&str>,
    cover_url: Option<&str>,
    description: Option<&str>,
    publisher: Option<&str>,
    published_date: Option<&str>,
    page_count: Option<i32>,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO books (title, author, isbn, asin, cover_url, description, \
         publisher, published_date, page_count, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'), datetime('now'))",
        rusqlite::params![
            title,
            author,
            isbn,
            asin,
            cover_url,
            description,
            publisher,
            published_date,
            page_count
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn list_books_db(conn: &rusqlite::Connection) -> Result<Vec<Book>, String> {
    let mut stmt = conn
        .prepare(&format!("{} ORDER BY b.updated_at DESC", BOOK_SELECT))
        .map_err(|e| e.to_string())?;
    let books = stmt
        .query_map([], row_to_book)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(books)
}

pub fn get_book_db(conn: &rusqlite::Connection, id: i64) -> Result<Book, String> {
    let mut stmt = conn
        .prepare(&format!("{} WHERE b.id = ?1", BOOK_SELECT))
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], row_to_book).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn update_book_db(
    conn: &rusqlite::Connection,
    id: i64,
    title: &str,
    author: Option<&str>,
    isbn: Option<&str>,
    asin: Option<&str>,
    cover_url: Option<&str>,
    description: Option<&str>,
    publisher: Option<&str>,
    published_date: Option<&str>,
    page_count: Option<i32>,
) -> Result<Book, String> {
    conn.execute(
        "UPDATE books SET title=?1, author=?2, isbn=?3, asin=?4, cover_url=?5, \
         description=?6, publisher=?7, published_date=?8, page_count=?9, \
         updated_at=datetime('now') WHERE id=?10",
        rusqlite::params![
            title,
            author,
            isbn,
            asin,
            cover_url,
            description,
            publisher,
            published_date,
            page_count,
            id
        ],
    )
    .map_err(|e| e.to_string())?;
    get_book_db(conn, id)
}

pub fn delete_book_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM books WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn set_rating_db(conn: &rusqlite::Connection, book_id: i64, score: i32) -> Result<(), String> {
    if !(1..=5).contains(&score) {
        return Err(format!("Rating must be between 1 and 5, got {score}"));
    }
    conn.execute(
        "INSERT INTO ratings (book_id, score, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET score=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, score],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn valid_date(d: &str) -> Result<(), String> {
    if d.len() != 10
        || d.as_bytes().get(4) != Some(&b'-')
        || d.as_bytes().get(7) != Some(&b'-')
        || !d[..4].chars().all(|c| c.is_ascii_digit())
        || !d[5..7].chars().all(|c| c.is_ascii_digit())
        || !d[8..10].chars().all(|c| c.is_ascii_digit())
    {
        return Err(format!("Invalid date format (expected YYYY-MM-DD): {d}"));
    }
    let month: u32 = d[5..7].parse().unwrap();
    let day: u32 = d[8..10].parse().unwrap();
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(format!("Invalid date (month/day out of range): {d}"));
    }
    Ok(())
}

pub fn set_reading_status_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    status: &str,
    started_at: Option<&str>,
    finished_at: Option<&str>,
) -> Result<(), String> {
    const VALID_STATUSES: &[&str] = &["want_to_read", "reading", "finished", "abandoned"];
    if !VALID_STATUSES.contains(&status) {
        return Err(format!("Invalid reading status: {status}"));
    }
    if let Some(d) = started_at {
        valid_date(d)?;
    }
    if let Some(d) = finished_at {
        valid_date(d)?;
    }

    let today = || -> Result<String, String> {
        conn.query_row("SELECT date('now')", [], |r| r.get(0))
            .map_err(|e| e.to_string())
    };

    match status {
        "reading" => {
            let started = match started_at {
                Some(d) => d.to_string(),
                None => today()?,
            };
            conn.execute(
                "INSERT INTO reading_status (book_id, status, started_at, finished_at, updated_at) \
                 VALUES (?1, ?2, ?3, NULL, datetime('now')) \
                 ON CONFLICT(book_id) DO UPDATE SET status=?2, \
                 started_at=?3, finished_at=NULL, updated_at=datetime('now')",
                rusqlite::params![book_id, status, started],
            )
        }
        "finished" | "abandoned" => {
            let finished = match finished_at {
                Some(d) => d.to_string(),
                None => today()?,
            };
            let started_override = started_at.map(|s| s.to_string());
            conn.execute(
                "INSERT INTO reading_status (book_id, status, started_at, finished_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, datetime('now')) \
                 ON CONFLICT(book_id) DO UPDATE SET status=?2, \
                 started_at=COALESCE(?3, reading_status.started_at), \
                 finished_at=?4, updated_at=datetime('now')",
                rusqlite::params![book_id, status, started_override, finished],
            )
        }
        _ => conn.execute(
            "INSERT INTO reading_status (book_id, status, started_at, finished_at, updated_at) \
                 VALUES (?1, ?2, NULL, NULL, datetime('now')) \
                 ON CONFLICT(book_id) DO UPDATE SET status=?2, \
                 started_at=NULL, finished_at=NULL, updated_at=datetime('now')",
            rusqlite::params![book_id, status],
        ),
    }
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_reading_dates_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    started_at: Option<&str>,
    finished_at: Option<&str>,
) -> Result<(), String> {
    if let Some(d) = started_at {
        valid_date(d)?;
    }
    if let Some(d) = finished_at {
        valid_date(d)?;
    }
    conn.execute(
        "UPDATE reading_status SET started_at=?2, finished_at=?3, updated_at=datetime('now') \
         WHERE book_id=?1",
        rusqlite::params![book_id, started_at, finished_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn import_kindle_books_db(
    conn: &mut rusqlite::Connection,
    books: &[KindleBook],
    covers_dir: Option<&Path>,
) -> Result<Vec<i64>, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if let Some(dir) = covers_dir {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut ids = Vec::new();
    for book in books {
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM books WHERE title = ?1)",
                [&book.title],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists {
            continue;
        }
        tx.execute(
            "INSERT INTO books (title, author, isbn, asin, publisher, description, \
             published_date, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))",
            rusqlite::params![
                book.title,
                book.author,
                book.isbn,
                book.asin,
                book.publisher,
                book.description,
                book.published_date,
            ],
        )
        .map_err(|e| e.to_string())?;
        let book_id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO reading_status (book_id, status, started_at, updated_at) \
             VALUES (?1, 'want_to_read', NULL, datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET status='want_to_read', \
             started_at=NULL, updated_at=datetime('now')",
            [book_id],
        )
        .map_err(|e| e.to_string())?;
        if let (Some(cover_b64), Some(dir)) = (&book.cover_data, covers_dir) {
            use base64::Engine;
            match base64::engine::general_purpose::STANDARD.decode(cover_b64) {
                Ok(bytes) => {
                    let cover_path = dir.join(format!("{}.jpg", book_id));
                    match cover_path.to_str() {
                        Some(cover_url) => {
                            if let Err(e) = fs::write(&cover_path, &bytes) {
                                eprintln!("warn: failed to write cover for '{}': {e}", book.title);
                            } else {
                                tx.execute(
                                    "UPDATE books SET cover_url = ?1 WHERE id = ?2",
                                    rusqlite::params![cover_url, book_id],
                                )
                                .map_err(|e| e.to_string())?;
                            }
                        }
                        None => {
                            eprintln!("warn: non-UTF-8 cover path for '{}'", book.title);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("warn: failed to decode cover for '{}': {e}", book.title);
                }
            }
        }
        ids.push(book_id);
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(ids)
}

pub fn upload_cover_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    source_path: &str,
) -> Result<String, String> {
    let source = Path::new(source_path);
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("jpg");

    let covers_dir = crate::data::db::covers_dir()?;

    // Remove any existing cover for this book
    if let Ok(entries) = fs::read_dir(&covers_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(&format!("{}.", book_id)) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let dest = covers_dir.join(format!("{}.{}", book_id, ext));
    fs::copy(source, &dest).map_err(|e| e.to_string())?;

    let dest_str = dest.to_string_lossy().to_string();
    conn.execute(
        "UPDATE books SET cover_url = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![dest_str, book_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(dest_str)
}

pub fn delete_book_with_covers_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    delete_book_db(conn, id)?;

    if let Ok(covers_dir) = crate::data::db::covers_dir() {
        if covers_dir.exists() {
            if let Ok(entries) = fs::read_dir(&covers_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with(&format!("{}.", id)) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    Ok(())
}

fn find_clippings_file(mount_path: &str) -> Option<std::path::PathBuf> {
    let docs_dir = crate::services::kindle::find_documents_dir(Path::new(mount_path))?;
    if let Ok(entries) = fs::read_dir(&docs_dir) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("My Clippings.txt")
            {
                return Some(entry.path());
            }
        }
    }
    None
}

#[derive(serde::Serialize)]
pub struct ClippingsInfo {
    pub exists: bool,
    pub count: usize,
}

pub fn check_clippings_exist(mount_path: &str) -> Result<ClippingsInfo, String> {
    match find_clippings_file(mount_path) {
        Some(path) => {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let count = crate::services::clippings::count_clipping_blocks(&content);
            Ok(ClippingsInfo {
                exists: true,
                count,
            })
        }
        None => Ok(ClippingsInfo {
            exists: false,
            count: 0,
        }),
    }
}

pub fn import_clippings_db(
    conn: &mut rusqlite::Connection,
    mount_path: &str,
) -> Result<usize, String> {
    let path = find_clippings_file(mount_path)
        .ok_or_else(|| "My Clippings.txt not found on device".to_string())?;
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let clippings = crate::services::clippings::parse_clippings(&content);

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut imported = 0usize;

    for clip in &clippings {
        let normalized_title = clip.title.trim().to_lowercase();
        let book_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM books WHERE LOWER(TRIM(title)) = ?1",
                [&normalized_title],
                |row| row.get(0),
            )
            .ok();

        let book_id = match book_id {
            Some(id) => id,
            None => continue,
        };

        if clip.location_start.is_none() {
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM highlights WHERE book_id = ?1 AND text = ?2 AND location_start IS NULL)",
                    rusqlite::params![book_id, clip.text],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            if exists {
                continue;
            }
        }

        let result = tx.execute(
            "INSERT INTO highlights (book_id, text, location_start, location_end, page, clip_type, clipped_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
             ON CONFLICT(book_id, text, location_start) DO NOTHING",
            rusqlite::params![
                book_id,
                clip.text,
                clip.location_start,
                clip.location_end,
                clip.page,
                clip.clip_type,
                clip.clipped_at,
            ],
        );

        match result {
            Ok(n) if n > 0 => imported += 1,
            _ => {}
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(imported)
}

pub fn enrich_book_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    meta: &BookMetadata,
) -> Result<(), String> {
    conn.execute(
        "UPDATE books SET \
         author = COALESCE(author, ?1), \
         isbn = COALESCE(isbn, ?2), \
         cover_url = COALESCE(?3, cover_url), \
         description = COALESCE(description, ?4), \
         publisher = COALESCE(publisher, ?5), \
         published_date = COALESCE(published_date, ?6), \
         page_count = COALESCE(page_count, ?7), \
         updated_at = datetime('now') WHERE id = ?8",
        rusqlite::params![
            meta.author,
            meta.isbn,
            meta.cover_url,
            meta.description,
            meta.publisher,
            meta.published_date,
            meta.page_count,
            book_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn list_highlights_db(
    conn: &rusqlite::Connection,
    book_id: i64,
) -> Result<Vec<Highlight>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, book_id, text, location_start, location_end, page, \
             clip_type, clipped_at, created_at \
             FROM highlights WHERE book_id = ?1 \
             ORDER BY location_start IS NULL, location_start, clipped_at",
        )
        .map_err(|e| e.to_string())?;

    let highlights = stmt
        .query_map([book_id], |row| {
            Ok(Highlight {
                id: row.get(0)?,
                book_id: row.get(1)?,
                text: row.get(2)?,
                location_start: row.get(3)?,
                location_end: row.get(4)?,
                page: row.get(5)?,
                clip_type: row.get(6)?,
                clipped_at: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(highlights)
}

pub fn search_highlights_db(
    conn: &rusqlite::Connection,
    query: &str,
) -> Result<Vec<HighlightSearchResult>, String> {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.book_id, h.text, h.location_start, h.location_end, h.page, \
             h.clip_type, h.clipped_at, h.created_at, b.title, b.author \
             FROM highlights h JOIN books b ON h.book_id = b.id \
             WHERE h.text LIKE '%' || ?1 || '%' ESCAPE '\\' \
             ORDER BY h.clipped_at DESC LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([&escaped], |row| {
            Ok(HighlightSearchResult {
                id: row.get(0)?,
                book_id: row.get(1)?,
                text: row.get(2)?,
                location_start: row.get(3)?,
                location_end: row.get(4)?,
                page: row.get(5)?,
                clip_type: row.get(6)?,
                clipped_at: row.get(7)?,
                created_at: row.get(8)?,
                book_title: row.get(9)?,
                book_author: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

pub fn create_shelf_db(conn: &rusqlite::Connection, name: &str) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Shelf name cannot be empty".to_string());
    }
    conn.execute(
        "INSERT INTO shelves (name) VALUES (?1)",
        rusqlite::params![name],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn list_shelves_db(conn: &rusqlite::Connection) -> Result<Vec<Shelf>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM shelves ORDER BY name")
        .map_err(|e| e.to_string())?;
    let shelves = stmt
        .query_map([], |row| {
            Ok(Shelf {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(shelves)
}

#[cfg(test)]
pub fn rename_shelf_db(conn: &rusqlite::Connection, id: i64, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Shelf name cannot be empty".to_string());
    }
    conn.execute(
        "UPDATE shelves SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
    .map_err(|e| e.to_string())?;
    if conn.changes() == 0 {
        return Err("Shelf not found".to_string());
    }
    Ok(())
}

#[cfg(test)]
pub fn delete_shelf_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM shelves WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    if conn.changes() == 0 {
        return Err("Shelf not found".to_string());
    }
    Ok(())
}

pub fn add_book_to_shelf_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    shelf_id: i64,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO book_shelves (book_id, shelf_id) VALUES (?1, ?2)",
        rusqlite::params![book_id, shelf_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_book_from_shelf_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    shelf_id: i64,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM book_shelves WHERE book_id = ?1 AND shelf_id = ?2",
        rusqlite::params![book_id, shelf_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_book_shelves_db(
    conn: &rusqlite::Connection,
    book_id: i64,
) -> Result<Vec<Shelf>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.name, s.created_at FROM shelves s \
             INNER JOIN book_shelves bs ON bs.shelf_id = s.id \
             WHERE bs.book_id = ?1 ORDER BY s.name",
        )
        .map_err(|e| e.to_string())?;
    let shelves = stmt
        .query_map([book_id], |row| {
            Ok(Shelf {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(shelves)
}

#[cfg(test)]
pub fn list_shelf_book_ids_db(
    conn: &rusqlite::Connection,
    shelf_id: i64,
) -> Result<Vec<i64>, String> {
    let mut stmt = conn
        .prepare("SELECT book_id FROM book_shelves WHERE shelf_id = ?1")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map([shelf_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(ids)
}

pub fn list_all_shelf_book_ids_db(conn: &rusqlite::Connection) -> Result<Vec<(i64, i64)>, String> {
    let mut stmt = conn
        .prepare("SELECT shelf_id, book_id FROM book_shelves")
        .map_err(|e| e.to_string())?;
    let pairs = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(pairs)
}

const DIARY_SELECT: &str = "SELECT d.id, d.book_id, b.title, b.author, b.cover_url, \
    d.body, d.rating, d.entry_date, d.created_at, d.updated_at \
    FROM diary_entries d \
    INNER JOIN books b ON b.id = d.book_id";

fn row_to_diary_entry(row: &rusqlite::Row) -> rusqlite::Result<DiaryEntry> {
    Ok(DiaryEntry {
        id: row.get(0)?,
        book_id: row.get(1)?,
        book_title: row.get(2)?,
        book_author: row.get(3)?,
        book_cover_url: row.get(4)?,
        body: row.get(5)?,
        rating: row.get(6)?,
        entry_date: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub fn create_diary_entry_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    body: Option<&str>,
    rating: Option<i32>,
    entry_date: &str,
) -> Result<DiaryEntry, String> {
    if let Some(r) = rating {
        if !(1..=5).contains(&r) {
            return Err(format!("Rating must be between 1 and 5, got {r}"));
        }
    }
    if entry_date.len() != 10
        || entry_date.as_bytes().get(4) != Some(&b'-')
        || entry_date.as_bytes().get(7) != Some(&b'-')
        || !entry_date[..4].chars().all(|c| c.is_ascii_digit())
        || !entry_date[5..7].chars().all(|c| c.is_ascii_digit())
        || !entry_date[8..].chars().all(|c| c.is_ascii_digit())
    {
        return Err(format!(
            "Invalid date format: {entry_date}, expected YYYY-MM-DD"
        ));
    }
    conn.execute(
        "INSERT INTO diary_entries (book_id, body, rating, entry_date, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
        rusqlite::params![book_id, body, rating, entry_date],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let mut stmt = conn
        .prepare(&format!("{} WHERE d.id = ?1", DIARY_SELECT))
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], row_to_diary_entry)
        .map_err(|e| e.to_string())
}

pub fn update_diary_entry_db(
    conn: &rusqlite::Connection,
    id: i64,
    body: Option<&str>,
    rating: Option<i32>,
    entry_date: &str,
) -> Result<(), String> {
    if let Some(r) = rating {
        if !(1..=5).contains(&r) {
            return Err(format!("Rating must be between 1 and 5, got {r}"));
        }
    }
    if entry_date.len() != 10
        || entry_date.as_bytes().get(4) != Some(&b'-')
        || entry_date.as_bytes().get(7) != Some(&b'-')
        || !entry_date[..4].chars().all(|c| c.is_ascii_digit())
        || !entry_date[5..7].chars().all(|c| c.is_ascii_digit())
        || !entry_date[8..].chars().all(|c| c.is_ascii_digit())
    {
        return Err(format!(
            "Invalid date format: {entry_date}, expected YYYY-MM-DD"
        ));
    }
    conn.execute(
        "UPDATE diary_entries SET body = ?1, rating = ?2, entry_date = ?3, \
         updated_at = datetime('now') WHERE id = ?4",
        rusqlite::params![body, rating, entry_date, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
pub fn delete_diary_entry_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM diary_entries WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_diary_entries_db(conn: &rusqlite::Connection) -> Result<Vec<DiaryEntry>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "{} ORDER BY d.entry_date DESC, d.id DESC",
            DIARY_SELECT
        ))
        .map_err(|e| e.to_string())?;
    let entries = stmt
        .query_map([], row_to_diary_entry)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(entries)
}

pub fn list_book_diary_entries_db(
    conn: &rusqlite::Connection,
    book_id: i64,
) -> Result<Vec<DiaryEntry>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "{} WHERE d.book_id = ?1 ORDER BY d.entry_date DESC, d.id DESC",
            DIARY_SELECT
        ))
        .map_err(|e| e.to_string())?;
    let entries = stmt
        .query_map([book_id], row_to_diary_entry)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(entries)
}

#[cfg(test)]
pub fn get_setting_db(conn: &rusqlite::Connection, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;
    let result = stmt.query_row([key], |row| row.get(0)).ok();
    Ok(result)
}

pub fn set_setting_db(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_all_settings_db(conn: &rusqlite::Connection) -> Result<HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let pairs = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<(String, String)>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(pairs.into_iter().collect())
}

pub fn get_reading_stats_db(conn: &rusqlite::Connection) -> Result<ReadingStats, String> {
    let total_books: usize = conn
        .query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let status_count = |status: &str| -> Result<usize, String> {
        conn.query_row(
            "SELECT COUNT(*) FROM reading_status WHERE status = ?1",
            [status],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())
    };

    let books_finished = status_count("finished")?;
    let books_reading = status_count("reading")?;
    let books_want_to_read = status_count("want_to_read")?;
    let books_abandoned = status_count("abandoned")?;

    let total_pages_read: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(b.page_count), 0) FROM books b \
             JOIN reading_status rs ON rs.book_id = b.id WHERE rs.status = 'finished'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let avg_rating: Option<f64> = conn
        .query_row("SELECT AVG(CAST(score AS REAL)) FROM ratings", [], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?;

    let total_diary_entries: usize = conn
        .query_row("SELECT COUNT(*) FROM diary_entries", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let total_highlights: usize = conn
        .query_row("SELECT COUNT(*) FROM highlights", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT strftime('%Y-%m', rs.finished_at) as month, COUNT(*) \
             FROM reading_status rs \
             WHERE rs.status = 'finished' AND rs.finished_at IS NOT NULL \
             GROUP BY month ORDER BY month DESC LIMIT 12",
        )
        .map_err(|e| e.to_string())?;
    let books_per_month: Vec<(String, usize)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT b.title, COALESCE(b.author, ''), r.score \
             FROM ratings r JOIN books b ON b.id = r.book_id \
             WHERE r.score = 5 ORDER BY r.id DESC LIMIT 10",
        )
        .map_err(|e| e.to_string())?;
    let top_rated_books: Vec<(String, String, i32)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut rating_distribution = [0usize; 5];
    let mut stmt = conn
        .prepare("SELECT score, COUNT(*) FROM ratings GROUP BY score")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(i32, usize)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for (score, count) in rows {
        if (1..=5).contains(&score) {
            rating_distribution[(score - 1) as usize] = count;
        }
    }

    let mut stmt = conn
        .prepare(
            "SELECT d.entry_date, 'diary', b.title \
             FROM diary_entries d JOIN books b ON b.id = d.book_id \
             ORDER BY d.entry_date DESC, d.id DESC LIMIT 10",
        )
        .map_err(|e| e.to_string())?;
    let recent_activity: Vec<(String, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(ReadingStats {
        total_books,
        books_finished,
        books_reading,
        books_want_to_read,
        books_abandoned,
        total_pages_read,
        avg_rating,
        total_diary_entries,
        total_highlights,
        books_per_month,
        top_rated_books,
        rating_distribution,
        recent_activity,
    })
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ImportStats {
    pub books_imported: usize,
    pub books_skipped: usize,
    pub entries_created: usize,
    pub shelves_created: usize,
}

pub fn import_goodreads_db(
    conn: &mut rusqlite::Connection,
    books: &[ParsedGoodreadsBook],
) -> Result<ImportStats, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut stats = ImportStats::default();

    let mut shelf_cache: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = tx
            .prepare("SELECT id, name FROM shelves")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, name) = row.map_err(|e| e.to_string())?;
            shelf_cache.insert(name, id);
        }
    }

    for book in books {
        let existing_id: Option<i64> = if let Some(ref isbn) = book.isbn {
            tx.query_row(
                "SELECT id FROM books WHERE isbn = ?1",
                rusqlite::params![isbn],
                |row| row.get(0),
            )
            .ok()
        } else {
            None
        };

        let existing_id = existing_id.or_else(|| {
            let author = book.author.as_deref().unwrap_or("");
            tx.query_row(
                "SELECT id FROM books WHERE LOWER(TRIM(title)) = ?1 AND \
                 COALESCE(LOWER(TRIM(author)), '') = ?2",
                rusqlite::params![
                    book.title.trim().to_lowercase(),
                    author.trim().to_lowercase()
                ],
                |row| row.get(0),
            )
            .ok()
        });

        if existing_id.is_some() {
            stats.books_skipped += 1;
            continue;
        }

        tx.execute(
            "INSERT INTO books (title, author, isbn, publisher, published_date, \
             page_count, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
            rusqlite::params![
                book.title,
                book.author,
                book.isbn,
                book.publisher,
                book.published_date,
                book.page_count,
            ],
        )
        .map_err(|e| e.to_string())?;
        let book_id = tx.last_insert_rowid();
        stats.books_imported += 1;

        let finished_at = if book.status == "finished" {
            book.date_read.as_deref()
        } else {
            None
        };

        tx.execute(
            "INSERT INTO reading_status (book_id, status, started_at, finished_at, updated_at) \
             VALUES (?1, ?2, NULL, ?3, datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET status=?2, finished_at=?3, updated_at=datetime('now')",
            rusqlite::params![book_id, book.status, finished_at],
        )
        .map_err(|e| e.to_string())?;

        if let Some(rating) = book.rating {
            tx.execute(
                "INSERT INTO ratings (book_id, score, created_at, updated_at) \
                 VALUES (?1, ?2, datetime('now'), datetime('now')) \
                 ON CONFLICT(book_id) DO UPDATE SET score=?2, updated_at=datetime('now')",
                rusqlite::params![book_id, rating],
            )
            .map_err(|e| e.to_string())?;
        }

        let entry_date = book.date_read.as_deref().or(book.date_added.as_deref());
        if let Some(date) = entry_date {
            if date.len() == 10 {
                tx.execute(
                    "INSERT INTO diary_entries (book_id, body, rating, entry_date, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
                    rusqlite::params![book_id, book.review, book.rating, date],
                )
                .map_err(|e| e.to_string())?;
                stats.entries_created += 1;
            }
        }

        for shelf_name in &book.shelves {
            let shelf_id = if let Some(id) = shelf_cache.get(shelf_name) {
                *id
            } else {
                tx.execute(
                    "INSERT INTO shelves (name) VALUES (?1)",
                    rusqlite::params![shelf_name],
                )
                .map_err(|e| e.to_string())?;
                let id = tx.last_insert_rowid();
                shelf_cache.insert(shelf_name.clone(), id);
                stats.shelves_created += 1;
                id
            };
            tx.execute(
                "INSERT OR IGNORE INTO book_shelves (book_id, shelf_id) VALUES (?1, ?2)",
                rusqlite::params![book_id, shelf_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(stats)
}

fn row_to_reading_goal(row: &rusqlite::Row) -> rusqlite::Result<ReadingGoal> {
    Ok(ReadingGoal {
        id: row.get(0)?,
        year: row.get(1)?,
        target_books: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn current_year_and_day_of_year() -> (i32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days_since_epoch = secs / 86400;
    let mut year = 1970i32;
    let mut remaining_days = days_since_epoch;
    loop {
        let days_in_year: u64 = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    (year, remaining_days as u32 + 1)
}

pub fn set_reading_goal_db(
    conn: &rusqlite::Connection,
    year: i32,
    target: i32,
) -> Result<ReadingGoal, String> {
    conn.execute(
        "INSERT INTO reading_goals (year, target_books, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(year) DO UPDATE SET target_books = ?2, updated_at = datetime('now')",
        rusqlite::params![year, target],
    )
    .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, year, target_books, created_at, updated_at FROM reading_goals WHERE year = ?1")
        .map_err(|e| e.to_string())?;
    stmt.query_row([year], row_to_reading_goal)
        .map_err(|e| e.to_string())
}

pub fn get_reading_goal_db(
    conn: &rusqlite::Connection,
    year: i32,
) -> Result<Option<ReadingGoal>, String> {
    let mut stmt = conn
        .prepare("SELECT id, year, target_books, created_at, updated_at FROM reading_goals WHERE year = ?1")
        .map_err(|e| e.to_string())?;
    let result = stmt
        .query_row([year], row_to_reading_goal)
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(result)
}

fn count_finished_books_in_year(conn: &rusqlite::Connection, year: i32) -> Result<i32, String> {
    let pattern = format!("{year}%");
    conn.query_row(
        "SELECT COUNT(*) FROM reading_status WHERE status = 'finished' AND finished_at LIKE ?1",
        [&pattern],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

fn build_progress(goal: ReadingGoal, books_finished: i32) -> ReadingGoalProgress {
    let (current_year, day_of_year) = current_year_and_day_of_year();
    let percent_complete = if goal.target_books > 0 {
        (books_finished as f64 / goal.target_books as f64) * 100.0
    } else {
        0.0
    };
    let on_track = if goal.year == current_year && goal.target_books > 0 {
        let expected_fraction = day_of_year as f64 / 365.0;
        let actual_fraction = books_finished as f64 / goal.target_books as f64;
        actual_fraction >= expected_fraction
    } else if goal.year < current_year {
        books_finished >= goal.target_books
    } else {
        true
    };
    ReadingGoalProgress {
        goal,
        books_finished,
        percent_complete,
        on_track,
    }
}

pub fn get_reading_goal_progress_db(
    conn: &rusqlite::Connection,
    year: i32,
) -> Result<Option<ReadingGoalProgress>, String> {
    let goal = match get_reading_goal_db(conn, year)? {
        Some(g) => g,
        None => return Ok(None),
    };
    let books_finished = count_finished_books_in_year(conn, year)?;
    Ok(Some(build_progress(goal, books_finished)))
}

pub fn list_reading_goals_db(
    conn: &rusqlite::Connection,
) -> Result<Vec<ReadingGoalProgress>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, year, target_books, created_at, updated_at \
             FROM reading_goals ORDER BY year DESC",
        )
        .map_err(|e| e.to_string())?;
    let goals: Vec<ReadingGoal> = stmt
        .query_map([], row_to_reading_goal)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for goal in goals {
        let year = goal.year;
        let books_finished = count_finished_books_in_year(conn, year)?;
        results.push(build_progress(goal, books_finished));
    }
    Ok(results)
}

fn row_to_book_note(row: &rusqlite::Row) -> rusqlite::Result<BookNote> {
    Ok(BookNote {
        id: row.get(0)?,
        book_id: row.get(1)?,
        content: row.get(2)?,
        color: row.get(3)?,
        pinned: row.get::<_, i32>(4)? != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

pub fn create_book_note_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    content: &str,
    color: &str,
) -> Result<BookNote, String> {
    conn.execute(
        "INSERT INTO book_notes (book_id, content, color) VALUES (?1, ?2, ?3)",
        rusqlite::params![book_id, content, color],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let mut stmt = conn
        .prepare(
            "SELECT id, book_id, content, color, pinned, created_at, updated_at \
             FROM book_notes WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], row_to_book_note)
        .map_err(|e| e.to_string())
}

pub fn list_book_notes_db(
    conn: &rusqlite::Connection,
    book_id: i64,
) -> Result<Vec<BookNote>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, book_id, content, color, pinned, created_at, updated_at \
             FROM book_notes WHERE book_id = ?1 \
             ORDER BY pinned DESC, created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let notes = stmt
        .query_map([book_id], row_to_book_note)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(notes)
}

pub fn update_book_note_db(
    conn: &rusqlite::Connection,
    id: i64,
    content: &str,
    color: &str,
    pinned: bool,
) -> Result<(), String> {
    conn.execute(
        "UPDATE book_notes SET content = ?1, color = ?2, pinned = ?3, \
         updated_at = datetime('now') WHERE id = ?4",
        rusqlite::params![content, color, pinned as i32, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_book_note_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM book_notes WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
pub fn create_collection_db(
    conn: &rusqlite::Connection,
    name: &str,
    description: Option<&str>,
) -> Result<Collection, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Collection name cannot be empty".to_string());
    }
    conn.execute(
        "INSERT INTO collections (name, description) VALUES (?1, ?2)",
        rusqlite::params![name, description],
    )
    .map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM collections WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(Collection {
        id,
        name: name.to_string(),
        description: description.map(String::from),
        book_count: 0,
        created_at,
    })
}

#[allow(dead_code)]
pub fn list_collections_db(conn: &rusqlite::Connection) -> Result<Vec<Collection>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.description, c.created_at, \
             (SELECT COUNT(*) FROM collection_books cb WHERE cb.collection_id = c.id) \
             FROM collections c ORDER BY c.updated_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let collections = stmt
        .query_map([], |row| {
            let count: i64 = row.get(4)?;
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                book_count: count as usize,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(collections)
}

#[allow(dead_code)]
pub fn get_collection_books_db(
    conn: &rusqlite::Connection,
    collection_id: i64,
) -> Result<Vec<Book>, String> {
    let sql = format!(
        "{} INNER JOIN collection_books cb ON cb.book_id = b.id \
         WHERE cb.collection_id = ?1 ORDER BY cb.position",
        BOOK_SELECT
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let books = stmt
        .query_map([collection_id], row_to_book)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(books)
}

#[allow(dead_code)]
pub fn add_book_to_collection_db(
    conn: &rusqlite::Connection,
    collection_id: i64,
    book_id: i64,
) -> Result<(), String> {
    let max_pos: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM collection_books WHERE collection_id = ?1",
            [collection_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO collection_books (collection_id, book_id, position) \
         VALUES (?1, ?2, ?3)",
        rusqlite::params![collection_id, book_id, max_pos + 1],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE collections SET updated_at = datetime('now') WHERE id = ?1",
        [collection_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
pub fn remove_book_from_collection_db(
    conn: &rusqlite::Connection,
    collection_id: i64,
    book_id: i64,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM collection_books WHERE collection_id = ?1 AND book_id = ?2",
        rusqlite::params![collection_id, book_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE collections SET updated_at = datetime('now') WHERE id = ?1",
        [collection_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
pub fn reorder_collection_db(
    conn: &rusqlite::Connection,
    collection_id: i64,
    book_ids: &[i64],
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "UPDATE collection_books SET position = ?1 \
             WHERE collection_id = ?2 AND book_id = ?3",
        )
        .map_err(|e| e.to_string())?;
    for (pos, book_id) in book_ids.iter().enumerate() {
        stmt.execute(rusqlite::params![pos as i64, collection_id, book_id])
            .map_err(|e| e.to_string())?;
    }
    conn.execute(
        "UPDATE collections SET updated_at = datetime('now') WHERE id = ?1",
        [collection_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
pub fn delete_collection_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM collections WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
pub fn search_books_fts_db(conn: &rusqlite::Connection, query: &str) -> Result<Vec<Book>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let fts_query: String = trimmed
        .split_whitespace()
        .map(|w| {
            let escaped = w.replace('"', "\"\"");
            format!("\"{escaped}\"*")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let fts_sql = format!(
        "{BOOK_SELECT} JOIN books_fts ON books_fts.rowid = b.id \
         WHERE books_fts MATCH ?1 ORDER BY books_fts.rank LIMIT 20"
    );

    if let Ok(mut stmt) = conn.prepare(&fts_sql) {
        if let Ok(books) = stmt
            .query_map(rusqlite::params![fts_query], row_to_book)
            .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
        {
            return Ok(books);
        }
    }

    let like_pattern = format!("%{trimmed}%");
    let fallback_sql = format!(
        "{BOOK_SELECT} WHERE b.title LIKE ?1 OR b.author LIKE ?1 \
         OR b.description LIKE ?1 ORDER BY b.updated_at DESC LIMIT 20"
    );
    let mut stmt = conn.prepare(&fallback_sql).map_err(|e| e.to_string())?;
    let books = stmt
        .query_map(rusqlite::params![like_pattern], row_to_book)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(books)
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
