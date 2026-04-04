use crate::kindle::KindleBook;
use crate::metadata::BookMetadata;
use crate::models::{Book, Highlight, Shelf};
use crate::AppState;
use std::fs;
use std::path::Path;
use tauri::Manager;
use tauri::State;

const BOOK_SELECT: &str = "SELECT b.id, b.title, b.author, b.isbn, b.asin, \
    b.cover_url, b.description, b.publisher, b.published_date, \
    b.page_count, b.created_at, b.updated_at, \
    r.score, rs.status, rv.body \
    FROM books b \
    LEFT JOIN ratings r ON r.book_id = b.id \
    LEFT JOIN reading_status rs ON rs.book_id = b.id \
    LEFT JOIN reviews rv ON rv.book_id = b.id";

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
        review: row.get(14)?,
    })
}

pub(crate) fn add_book_db(
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
) -> Result<i64, rusqlite::Error> {
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
    )?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_books_db(conn: &rusqlite::Connection) -> Result<Vec<Book>, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY b.updated_at DESC", BOOK_SELECT))?;
    let books = stmt
        .query_map([], row_to_book)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(books)
}

pub(crate) fn get_book_db(conn: &rusqlite::Connection, id: i64) -> Result<Book, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("{} WHERE b.id = ?1", BOOK_SELECT))?;
    stmt.query_row([id], row_to_book)
}

pub(crate) fn update_book_db(
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
) -> Result<Book, rusqlite::Error> {
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
    )?;
    get_book_db(conn, id)
}

pub(crate) fn delete_book_db(conn: &rusqlite::Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM books WHERE id = ?1", [id])?;
    Ok(())
}

pub(crate) fn set_rating_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    score: i32,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO ratings (book_id, score, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET score=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, score],
    )?;
    Ok(())
}

pub(crate) fn set_reading_status_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    status: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO reading_status (book_id, status, updated_at) \
         VALUES (?1, ?2, datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET status=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, status],
    )?;
    Ok(())
}

pub(crate) fn save_review_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    body: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO reviews (book_id, body, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET body=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, body],
    )?;
    Ok(())
}

pub(crate) fn import_kindle_books_db(
    conn: &rusqlite::Connection,
    books: &[KindleBook],
) -> Result<Vec<i64>, rusqlite::Error> {
    let mut ids = Vec::new();
    for book in books {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM books WHERE title = ?1)",
            [&book.title],
            |row| row.get(0),
        )?;
        if exists {
            continue;
        }
        conn.execute(
            "INSERT INTO books (title, author, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            rusqlite::params![book.title, book.author],
        )?;
        let book_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO reading_status (book_id, status, updated_at) \
             VALUES (?1, 'reading', datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET status='reading', updated_at=datetime('now')",
            [book_id],
        )?;
        ids.push(book_id);
    }
    Ok(ids)
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_book(
    state: State<AppState>,
    title: String,
    author: Option<String>,
    isbn: Option<String>,
    asin: Option<String>,
    cover_url: Option<String>,
    description: Option<String>,
    publisher: Option<String>,
    published_date: Option<String>,
    page_count: Option<i32>,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    add_book_db(
        &db,
        &title,
        author.as_deref(),
        isbn.as_deref(),
        asin.as_deref(),
        cover_url.as_deref(),
        description.as_deref(),
        publisher.as_deref(),
        published_date.as_deref(),
        page_count,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_books(state: State<AppState>) -> Result<Vec<Book>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_books_db(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_book(state: State<AppState>, id: i64) -> Result<Book, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    get_book_db(&db, id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_book(
    state: State<AppState>,
    id: i64,
    title: String,
    author: Option<String>,
    isbn: Option<String>,
    asin: Option<String>,
    cover_url: Option<String>,
    description: Option<String>,
    publisher: Option<String>,
    published_date: Option<String>,
    page_count: Option<i32>,
) -> Result<Book, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    update_book_db(
        &db,
        id,
        &title,
        author.as_deref(),
        isbn.as_deref(),
        asin.as_deref(),
        cover_url.as_deref(),
        description.as_deref(),
        publisher.as_deref(),
        published_date.as_deref(),
        page_count,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_book(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    delete_book_db(&db, id).map_err(|e| e.to_string())?;

    // Clean up cover files
    let covers_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("covers");
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

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_rating(state: State<AppState>, book_id: i64, score: i32) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    set_rating_db(&db, book_id, score).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_reading_status(
    state: State<AppState>,
    book_id: i64,
    status: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    set_reading_status_db(&db, book_id, &status).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_review(state: State<AppState>, book_id: i64, body: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    save_review_db(&db, book_id, &body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lookup_isbn(isbn: String) -> Result<BookMetadata, String> {
    crate::metadata::lookup(&isbn).await
}

#[tauri::command]
pub async fn search_covers(query: String) -> Result<Vec<BookMetadata>, String> {
    crate::metadata::search_covers(&query).await
}

#[tauri::command]
pub fn detect_kindle() -> Result<Option<String>, String> {
    Ok(crate::kindle::detect_kindle())
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_kindle_books(mount_path: String) -> Result<Vec<KindleBook>, String> {
    Ok(crate::kindle::list_kindle_books(&mount_path))
}

#[tauri::command]
pub fn import_kindle_books(
    _app_handle: tauri::AppHandle,
    state: State<AppState>,
    books: Vec<KindleBook>,
) -> Result<Vec<i64>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    import_kindle_books_db(&db, &books).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn upload_cover(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    book_id: i64,
    source_path: String,
) -> Result<String, String> {
    let source = Path::new(&source_path);
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("jpg");

    let covers_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("covers");
    fs::create_dir_all(&covers_dir).map_err(|e| e.to_string())?;

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
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE books SET cover_url = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![dest_str, book_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(dest_str)
}

fn find_clippings_file(mount_path: &str) -> Option<std::path::PathBuf> {
    let docs_dir = crate::kindle::find_documents_dir(Path::new(mount_path))?;
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

#[tauri::command(rename_all = "snake_case")]
pub fn check_clippings_exist(mount_path: String) -> Result<ClippingsInfo, String> {
    match find_clippings_file(&mount_path) {
        Some(path) => {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let count = crate::clippings::count_clipping_blocks(&content);
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

#[tauri::command(rename_all = "snake_case")]
pub fn import_clippings(state: State<AppState>, mount_path: String) -> Result<usize, String> {
    let path = find_clippings_file(&mount_path)
        .ok_or_else(|| "My Clippings.txt not found on device".to_string())?;
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let clippings = crate::clippings::parse_clippings(&content);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut imported = 0usize;

    db.execute_batch("BEGIN").map_err(|e| e.to_string())?;

    for clip in &clippings {
        let normalized_title = clip.title.trim().to_lowercase();
        let book_id: Option<i64> = db
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

        // SQLite treats NULL != NULL, so ON CONFLICT won't catch duplicates
        // when location_start is NULL. Check manually in that case.
        if clip.location_start.is_none() {
            let exists: bool = db
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

        let result = db.execute(
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

    db.execute_batch("COMMIT").map_err(|e| e.to_string())?;

    Ok(imported)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn enrich_book(state: State<'_, AppState>, book_id: i64) -> Result<(), String> {
    let (title, author, isbn, cover_url, description, publisher, published_date, page_count) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db
            .prepare(
                "SELECT title, author, isbn, cover_url, description, publisher, \
                 published_date, page_count FROM books WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;
        stmt.query_row([book_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<i32>>(7)?,
            ))
        })
        .map_err(|e| e.to_string())?
    };

    let meta = crate::metadata::search_by_title(&title, author.as_deref()).await?;

    // Only fill NULL columns
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE books SET isbn = ?1, cover_url = ?2, description = ?3, \
         publisher = ?4, published_date = ?5, page_count = ?6, \
         updated_at = datetime('now') WHERE id = ?7",
        rusqlite::params![
            if isbn.is_none() { meta.isbn } else { isbn },
            if cover_url.is_none() {
                meta.cover_url
            } else {
                cover_url
            },
            if description.is_none() {
                meta.description
            } else {
                description
            },
            if publisher.is_none() {
                meta.publisher
            } else {
                publisher
            },
            if published_date.is_none() {
                meta.published_date
            } else {
                published_date
            },
            if page_count.is_none() {
                meta.page_count
            } else {
                page_count
            },
            book_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_highlights(state: State<AppState>, book_id: i64) -> Result<Vec<Highlight>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
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

#[tauri::command]
pub fn create_shelf(state: State<AppState>, name: String) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO shelves (name) VALUES (?1)",
        rusqlite::params![name],
    )
    .map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

#[tauri::command]
pub fn list_shelves(state: State<AppState>) -> Result<Vec<Shelf>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
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

#[tauri::command]
pub fn rename_shelf(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE shelves SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_shelf(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM shelves WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_book_to_shelf(
    state: State<AppState>,
    book_id: i64,
    shelf_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR IGNORE INTO book_shelves (book_id, shelf_id) VALUES (?1, ?2)",
        rusqlite::params![book_id, shelf_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_book_from_shelf(
    state: State<AppState>,
    book_id: i64,
    shelf_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM book_shelves WHERE book_id = ?1 AND shelf_id = ?2",
        rusqlite::params![book_id, shelf_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_book_shelves(state: State<AppState>, book_id: i64) -> Result<Vec<Shelf>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
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

#[tauri::command(rename_all = "snake_case")]
pub fn list_shelf_book_ids(state: State<AppState>, shelf_id: i64) -> Result<Vec<i64>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT book_id FROM book_shelves WHERE shelf_id = ?1")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map([shelf_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kindle::KindleBook;

    fn test_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_add_and_list_books() {
        let conn = test_conn();
        let id = add_book_db(
            &conn,
            "Test Book",
            Some("Test Author"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let books = list_books_db(&conn).unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, id);
        assert_eq!(books[0].title, "Test Book");
        assert_eq!(books[0].author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_get_book() {
        let conn = test_conn();
        let id = add_book_db(
            &conn,
            "Get Me",
            Some("Author"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let book = get_book_db(&conn, id).unwrap();
        assert_eq!(book.title, "Get Me");
        assert_eq!(book.author, Some("Author".to_string()));
        assert_eq!(book.rating, None);
        assert_eq!(book.status, None);
        assert_eq!(book.review, None);
    }

    #[test]
    fn test_update_book() {
        let conn = test_conn();
        let id = add_book_db(
            &conn,
            "Old Title",
            Some("Old Author"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let book = update_book_db(
            &conn,
            id,
            "New Title",
            Some("New Author"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(book.title, "New Title");
        assert_eq!(book.author, Some("New Author".to_string()));
    }

    #[test]
    fn test_delete_book_cascades() {
        let conn = test_conn();
        let id = add_book_db(
            &conn, "Doomed", None, None, None, None, None, None, None, None,
        )
        .unwrap();

        set_rating_db(&conn, id, 4).unwrap();
        set_reading_status_db(&conn, id, "reading").unwrap();
        save_review_db(&conn, id, "Great").unwrap();

        delete_book_db(&conn, id).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ratings WHERE book_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM reading_status WHERE book_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM reviews WHERE book_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_set_rating() {
        let conn = test_conn();
        let id = add_book_db(
            &conn, "Rated", None, None, None, None, None, None, None, None,
        )
        .unwrap();

        set_rating_db(&conn, id, 3).unwrap();
        let book = get_book_db(&conn, id).unwrap();
        assert_eq!(book.rating, Some(3));

        set_rating_db(&conn, id, 5).unwrap();
        let book = get_book_db(&conn, id).unwrap();
        assert_eq!(book.rating, Some(5));
    }

    #[test]
    fn test_rating_constraint() {
        let conn = test_conn();
        let id = add_book_db(
            &conn,
            "Bad Rating",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert!(set_rating_db(&conn, id, 0).is_err());
        assert!(set_rating_db(&conn, id, 6).is_err());
    }

    #[test]
    fn test_set_reading_status() {
        let conn = test_conn();
        let id = add_book_db(
            &conn,
            "Status Book",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        set_reading_status_db(&conn, id, "reading").unwrap();
        let book = get_book_db(&conn, id).unwrap();
        assert_eq!(book.status, Some("reading".to_string()));

        set_reading_status_db(&conn, id, "finished").unwrap();
        let book = get_book_db(&conn, id).unwrap();
        assert_eq!(book.status, Some("finished".to_string()));
    }

    #[test]
    fn test_invalid_reading_status() {
        let conn = test_conn();
        let id = add_book_db(
            &conn,
            "Invalid Status",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert!(set_reading_status_db(&conn, id, "bogus").is_err());
    }

    #[test]
    fn test_save_review() {
        let conn = test_conn();
        let id = add_book_db(
            &conn, "Reviewed", None, None, None, None, None, None, None, None,
        )
        .unwrap();

        save_review_db(&conn, id, "Amazing book").unwrap();
        let book = get_book_db(&conn, id).unwrap();
        assert_eq!(book.review, Some("Amazing book".to_string()));

        save_review_db(&conn, id, "Updated review").unwrap();
        let book = get_book_db(&conn, id).unwrap();
        assert_eq!(book.review, Some("Updated review".to_string()));
    }

    #[test]
    fn test_import_kindle_skips_duplicates() {
        let conn = test_conn();
        add_book_db(
            &conn,
            "Existing Book",
            Some("Author"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let kindle_books = vec![
            KindleBook {
                filename: "existing.mobi".to_string(),
                path: "/kindle/existing.mobi".to_string(),
                title: "Existing Book".to_string(),
                author: Some("Author".to_string()),
                asin: None,
                isbn: None,
                publisher: None,
                description: None,
                published_date: None,
                language: None,
                cover_data: None,
                cde_type: None,
                extension: "mobi".to_string(),
                size_bytes: 1000,
            },
            KindleBook {
                filename: "new.mobi".to_string(),
                path: "/kindle/new.mobi".to_string(),
                title: "New Book".to_string(),
                author: Some("New Author".to_string()),
                asin: None,
                isbn: None,
                publisher: None,
                description: None,
                published_date: None,
                language: None,
                cover_data: None,
                cde_type: None,
                extension: "mobi".to_string(),
                size_bytes: 2000,
            },
        ];

        let ids = import_kindle_books_db(&conn, &kindle_books).unwrap();
        assert_eq!(ids.len(), 1, "should only import the new book");

        let new_book = get_book_db(&conn, ids[0]).unwrap();
        assert_eq!(new_book.title, "New Book");
        assert_eq!(
            new_book.status,
            Some("reading".to_string()),
            "imported books get 'reading' status"
        );

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2);
    }

    // --- Shelf helper functions (mirror the tauri commands but take &Connection) ---

    fn create_shelf_db(conn: &rusqlite::Connection, name: &str) -> Result<i64, rusqlite::Error> {
        conn.execute(
            "INSERT INTO shelves (name) VALUES (?1)",
            rusqlite::params![name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn list_shelves_db(conn: &rusqlite::Connection) -> Result<Vec<Shelf>, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM shelves ORDER BY name")?;
        let shelves = stmt
            .query_map([], |row| {
                Ok(Shelf {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(shelves)
    }

    fn rename_shelf_db(
        conn: &rusqlite::Connection,
        id: i64,
        name: &str,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE shelves SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        )?;
        Ok(())
    }

    fn delete_shelf_db(conn: &rusqlite::Connection, id: i64) -> Result<(), rusqlite::Error> {
        conn.execute("DELETE FROM shelves WHERE id = ?1", [id])?;
        Ok(())
    }

    fn add_book_to_shelf_db(
        conn: &rusqlite::Connection,
        book_id: i64,
        shelf_id: i64,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT OR IGNORE INTO book_shelves (book_id, shelf_id) VALUES (?1, ?2)",
            rusqlite::params![book_id, shelf_id],
        )?;
        Ok(())
    }

    fn remove_book_from_shelf_db(
        conn: &rusqlite::Connection,
        book_id: i64,
        shelf_id: i64,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "DELETE FROM book_shelves WHERE book_id = ?1 AND shelf_id = ?2",
            rusqlite::params![book_id, shelf_id],
        )?;
        Ok(())
    }

    fn list_book_shelves_db(
        conn: &rusqlite::Connection,
        book_id: i64,
    ) -> Result<Vec<Shelf>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.name, s.created_at FROM shelves s \
             INNER JOIN book_shelves bs ON bs.shelf_id = s.id \
             WHERE bs.book_id = ?1 ORDER BY s.name",
        )?;
        let shelves = stmt
            .query_map([book_id], |row| {
                Ok(Shelf {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(shelves)
    }

    fn list_shelf_book_ids_db(
        conn: &rusqlite::Connection,
        shelf_id: i64,
    ) -> Result<Vec<i64>, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT book_id FROM book_shelves WHERE shelf_id = ?1")?;
        let ids = stmt
            .query_map([shelf_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ids)
    }

    #[test]
    fn test_create_and_list_shelves() {
        let conn = test_conn();
        let id = create_shelf_db(&conn, "Fiction").unwrap();

        let shelves = list_shelves_db(&conn).unwrap();
        assert_eq!(shelves.len(), 1);
        assert_eq!(shelves[0].id, id);
        assert_eq!(shelves[0].name, "Fiction");
    }

    #[test]
    fn test_rename_shelf() {
        let conn = test_conn();
        let id = create_shelf_db(&conn, "Old Name").unwrap();

        rename_shelf_db(&conn, id, "New Name").unwrap();

        let shelves = list_shelves_db(&conn).unwrap();
        assert_eq!(shelves.len(), 1);
        assert_eq!(shelves[0].name, "New Name");
    }

    #[test]
    fn test_delete_shelf() {
        let conn = test_conn();
        let id = create_shelf_db(&conn, "Temporary").unwrap();
        assert_eq!(list_shelves_db(&conn).unwrap().len(), 1);

        delete_shelf_db(&conn, id).unwrap();
        assert_eq!(list_shelves_db(&conn).unwrap().len(), 0);
    }

    #[test]
    fn test_add_book_to_shelf_and_query() {
        let conn = test_conn();
        let book_id = add_book_db(
            &conn, "Shelf Book", None, None, None, None, None, None, None, None,
        )
        .unwrap();
        let shelf_id = create_shelf_db(&conn, "Favorites").unwrap();

        add_book_to_shelf_db(&conn, book_id, shelf_id).unwrap();

        let shelves = list_book_shelves_db(&conn, book_id).unwrap();
        assert_eq!(shelves.len(), 1);
        assert_eq!(shelves[0].name, "Favorites");

        let ids = list_shelf_book_ids_db(&conn, shelf_id).unwrap();
        assert_eq!(ids, vec![book_id]);
    }

    #[test]
    fn test_remove_book_from_shelf() {
        let conn = test_conn();
        let book_id = add_book_db(
            &conn, "Removable", None, None, None, None, None, None, None, None,
        )
        .unwrap();
        let shelf_id = create_shelf_db(&conn, "To Remove").unwrap();

        add_book_to_shelf_db(&conn, book_id, shelf_id).unwrap();
        assert_eq!(list_shelf_book_ids_db(&conn, shelf_id).unwrap().len(), 1);

        remove_book_from_shelf_db(&conn, book_id, shelf_id).unwrap();
        assert_eq!(list_shelf_book_ids_db(&conn, shelf_id).unwrap().len(), 0);
        assert_eq!(list_book_shelves_db(&conn, book_id).unwrap().len(), 0);
    }

    #[test]
    fn test_duplicate_shelf_name_errors() {
        let conn = test_conn();
        create_shelf_db(&conn, "Unique").unwrap();
        assert!(create_shelf_db(&conn, "Unique").is_err());
    }

    #[test]
    fn test_delete_book_cascades_book_shelves() {
        let conn = test_conn();
        let book_id = add_book_db(
            &conn, "Cascade Book", None, None, None, None, None, None, None, None,
        )
        .unwrap();
        let shelf_id = create_shelf_db(&conn, "Cascade Shelf").unwrap();
        add_book_to_shelf_db(&conn, book_id, shelf_id).unwrap();

        delete_book_db(&conn, book_id).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM book_shelves WHERE book_id = ?1",
                [book_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_delete_shelf_cascades_book_shelves() {
        let conn = test_conn();
        let book_id = add_book_db(
            &conn, "Stays", None, None, None, None, None, None, None, None,
        )
        .unwrap();
        let shelf_id = create_shelf_db(&conn, "Goes Away").unwrap();
        add_book_to_shelf_db(&conn, book_id, shelf_id).unwrap();

        delete_shelf_db(&conn, shelf_id).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM book_shelves WHERE shelf_id = ?1",
                [shelf_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
        // Book itself should still exist
        assert!(get_book_db(&conn, book_id).is_ok());
    }

    #[test]
    fn test_import_clippings_null_location_dedup() {
        let conn = test_conn();

        conn.execute(
            "INSERT INTO books (title, author, created_at, updated_at) VALUES ('Test Book', 'Author', datetime('now'), datetime('now'))",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO highlights (book_id, text, location_start, clip_type, created_at) VALUES (1, 'some text', NULL, 'bookmark', datetime('now'))",
            [],
        ).unwrap();

        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM highlights WHERE book_id = 1 AND text = 'some text' AND location_start IS NULL)",
            [],
            |row| row.get(0),
        ).unwrap();
        assert!(
            exists,
            "Dedup check should detect existing NULL-location highlight"
        );

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM highlights WHERE book_id = 1 AND text = 'some text'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
