use crate::data::models::{Book, DiaryEntry, Highlight, HighlightSearchResult, Shelf};
use crate::services::goodreads::GoodreadsBook;
use crate::services::kindle::KindleBook;
use crate::services::metadata::BookMetadata;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

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
    let id = conn.last_insert_rowid();
    info!(id, title, "book added");
    Ok(id)
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

#[tracing::instrument(skip_all, fields(book_count = books.len()), err)]
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
                                warn!(title = %book.title, "failed to write cover: {e}");
                            } else {
                                tx.execute(
                                    "UPDATE books SET cover_url = ?1 WHERE id = ?2",
                                    rusqlite::params![cover_url, book_id],
                                )
                                .map_err(|e| e.to_string())?;
                            }
                        }
                        None => {
                            warn!(title = %book.title, "non-UTF-8 cover path");
                        }
                    }
                }
                Err(e) => {
                    warn!(title = %book.title, "failed to decode cover: {e}");
                }
            }
        }
        ids.push(book_id);
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(ids)
}

#[derive(Debug)]
pub struct ImportResult {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub new_book_ids: Vec<i64>,
}

pub fn count_goodreads_duplicates_db(
    conn: &rusqlite::Connection,
    books: &[GoodreadsBook],
) -> Result<usize, String> {
    let mut stmt = conn
        .prepare("SELECT EXISTS(SELECT 1 FROM books WHERE LOWER(TRIM(title)) = LOWER(TRIM(?1)))")
        .map_err(|e| e.to_string())?;
    let mut count = 0;
    for book in books {
        let exists: bool = stmt
            .query_row([&book.title], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if exists {
            count += 1;
        }
    }
    Ok(count)
}

#[tracing::instrument(skip_all, fields(book_count = books.len()), err)]
pub fn import_goodreads_books_db(
    conn: &mut rusqlite::Connection,
    books: &[GoodreadsBook],
) -> Result<ImportResult, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut new_book_ids = Vec::new();
    let mut skipped_count = 0;

    for book in books {
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM books WHERE LOWER(TRIM(title)) = LOWER(TRIM(?1)))",
                [&book.title],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists {
            skipped_count += 1;
            continue;
        }

        // Pick best ISBN available
        let isbn = book.isbn.as_deref().or(book.isbn13.as_deref());

        tx.execute(
            "INSERT INTO books (title, author, isbn, publisher, published_date, page_count, \
             created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
            rusqlite::params![
                book.title,
                book.author,
                isbn,
                book.publisher,
                book.published_year,
                book.page_count,
            ],
        )
        .map_err(|e| e.to_string())?;
        let book_id = tx.last_insert_rowid();

        // Reading status
        let finished_at = if book.status == "finished" {
            book.date_read.as_deref()
        } else {
            None
        };
        let started_at = if book.status == "reading" {
            book.date_added.as_deref()
        } else {
            None
        };
        tx.execute(
            "INSERT OR REPLACE INTO reading_status (book_id, status, started_at, finished_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            rusqlite::params![book_id, book.status, started_at, finished_at],
        )
        .map_err(|e| e.to_string())?;

        // Rating
        if let Some(score) = book.rating {
            tx.execute(
                "INSERT OR REPLACE INTO ratings (book_id, score, created_at, updated_at) \
                 VALUES (?1, ?2, datetime('now'), datetime('now'))",
                rusqlite::params![book_id, score],
            )
            .map_err(|e| e.to_string())?;
        }

        // Review
        if let Some(ref text) = book.review_text {
            if !text.is_empty() {
                tx.execute(
                    "INSERT OR IGNORE INTO reviews (book_id, body, created_at, updated_at) \
                     VALUES (?1, ?2, datetime('now'), datetime('now'))",
                    rusqlite::params![book_id, text],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        // Bookshelves
        for shelf_name in &book.bookshelves {
            tx.execute(
                "INSERT OR IGNORE INTO shelves (name) VALUES (?1)",
                rusqlite::params![shelf_name],
            )
            .map_err(|e| e.to_string())?;
            let shelf_id: i64 = tx
                .query_row(
                    "SELECT id FROM shelves WHERE name = ?1",
                    rusqlite::params![shelf_name],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT OR IGNORE INTO book_shelves (book_id, shelf_id) VALUES (?1, ?2)",
                rusqlite::params![book_id, shelf_id],
            )
            .map_err(|e| e.to_string())?;
        }

        new_book_ids.push(book_id);
    }

    tx.commit().map_err(|e| e.to_string())?;
    let imported_count = new_book_ids.len();
    info!(imported_count, skipped_count, "goodreads import complete");
    Ok(ImportResult {
        imported_count,
        skipped_count,
        new_book_ids,
    })
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
    info!(book_id = id, "book deleted");

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

fn map_highlight_row(row: &rusqlite::Row) -> rusqlite::Result<HighlightSearchResult> {
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
        book_rating: row.get(11)?,
    })
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
             h.clip_type, h.clipped_at, h.created_at, b.title, b.author, r.score \
             FROM highlights h JOIN books b ON h.book_id = b.id \
             LEFT JOIN ratings r ON r.book_id = h.book_id \
             WHERE h.text LIKE '%' || ?1 || '%' ESCAPE '\\' \
             ORDER BY h.clipped_at DESC LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([&escaped], map_highlight_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

pub fn list_all_highlights_db(
    conn: &rusqlite::Connection,
) -> Result<Vec<HighlightSearchResult>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.book_id, h.text, h.location_start, h.location_end, h.page, \
             h.clip_type, h.clipped_at, h.created_at, b.title, b.author, r.score \
             FROM highlights h JOIN books b ON h.book_id = b.id \
             LEFT JOIN ratings r ON r.book_id = h.book_id \
             ORDER BY h.clipped_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([], map_highlight_row)
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

fn sync_book_rating_if_latest(
    conn: &rusqlite::Connection,
    entry_id: i64,
    book_id: i64,
    rating: i32,
) -> Result<(), String> {
    let latest_id: i64 = conn
        .query_row(
            "SELECT id FROM diary_entries WHERE book_id = ?1 ORDER BY entry_date DESC, id DESC LIMIT 1",
            [book_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if latest_id == entry_id {
        set_rating_db(conn, book_id, rating)?;
    }
    Ok(())
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
    if let Some(r) = rating {
        sync_book_rating_if_latest(conn, id, book_id, r)?;
    }
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
    if let Some(r) = rating {
        let book_id: i64 = conn
            .query_row(
                "SELECT book_id FROM diary_entries WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        sync_book_rating_if_latest(conn, id, book_id, r)?;
    }
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
#[path = "commands_tests.rs"]
mod tests;
