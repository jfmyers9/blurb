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

#[allow(clippy::too_many_arguments)]
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

pub(crate) fn list_books_db(conn: &rusqlite::Connection) -> Result<Vec<Book>, String> {
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

pub(crate) fn get_book_db(conn: &rusqlite::Connection, id: i64) -> Result<Book, String> {
    let mut stmt = conn
        .prepare(&format!("{} WHERE b.id = ?1", BOOK_SELECT))
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], row_to_book).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
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

pub(crate) fn delete_book_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM books WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn set_rating_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    score: i32,
) -> Result<(), String> {
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

pub(crate) fn set_reading_status_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    status: &str,
) -> Result<(), String> {
    const VALID_STATUSES: &[&str] = &["want_to_read", "reading", "finished", "abandoned"];
    if !VALID_STATUSES.contains(&status) {
        return Err(format!("Invalid reading status: {status}"));
    }
    conn.execute(
        "INSERT INTO reading_status (book_id, status, updated_at) \
         VALUES (?1, ?2, datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET status=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, status],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn save_review_db(
    conn: &rusqlite::Connection,
    book_id: i64,
    body: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO reviews (book_id, body, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET body=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, body],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn import_kindle_books_db(
    conn: &mut rusqlite::Connection,
    books: &[KindleBook],
) -> Result<Vec<i64>, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
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
            "INSERT INTO books (title, author, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            rusqlite::params![book.title, book.author],
        )
        .map_err(|e| e.to_string())?;
        let book_id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO reading_status (book_id, status, updated_at) \
             VALUES (?1, 'reading', datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET status='reading', updated_at=datetime('now')",
            [book_id],
        )
        .map_err(|e| e.to_string())?;
        ids.push(book_id);
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(ids)
}

#[allow(clippy::too_many_arguments)]
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
}

#[tauri::command]
pub fn list_books(state: State<AppState>) -> Result<Vec<Book>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_books_db(&db)
}

#[tauri::command]
pub fn get_book(state: State<AppState>, id: i64) -> Result<Book, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    get_book_db(&db, id)
}

#[allow(clippy::too_many_arguments)]
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
}

#[tauri::command]
pub fn delete_book(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    delete_book_db(&db, id)?;

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
    set_rating_db(&db, book_id, score)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_reading_status(
    state: State<AppState>,
    book_id: i64,
    status: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    set_reading_status_db(&db, book_id, &status)
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_review(state: State<AppState>, book_id: i64, body: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    save_review_db(&db, book_id, &body)
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
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    import_kindle_books_db(&mut db, &books)
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

    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let tx = db.transaction().map_err(|e| e.to_string())?;
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

        // SQLite treats NULL != NULL, so ON CONFLICT won't catch duplicates
        // when location_start is NULL. Check manually in that case.
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

#[tauri::command(rename_all = "snake_case")]
pub async fn enrich_book(state: State<'_, AppState>, book_id: i64) -> Result<(), String> {
    let (title, author) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT title, author FROM books WHERE id = ?1",
            [book_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .map_err(|e| e.to_string())?
    };

    let meta = crate::metadata::search_by_title(&title, author.as_deref()).await?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE books SET \
         isbn = COALESCE(isbn, ?1), \
         cover_url = COALESCE(cover_url, ?2), \
         description = COALESCE(description, ?3), \
         publisher = COALESCE(publisher, ?4), \
         published_date = COALESCE(published_date, ?5), \
         page_count = COALESCE(page_count, ?6), \
         updated_at = datetime('now') WHERE id = ?7",
        rusqlite::params![
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

pub(crate) fn create_shelf_db(conn: &rusqlite::Connection, name: &str) -> Result<i64, String> {
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

pub(crate) fn list_shelves_db(conn: &rusqlite::Connection) -> Result<Vec<Shelf>, String> {
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

pub(crate) fn rename_shelf_db(
    conn: &rusqlite::Connection,
    id: i64,
    name: &str,
) -> Result<(), String> {
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

pub(crate) fn delete_shelf_db(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM shelves WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    if conn.changes() == 0 {
        return Err("Shelf not found".to_string());
    }
    Ok(())
}

pub(crate) fn add_book_to_shelf_db(
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

pub(crate) fn remove_book_from_shelf_db(
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

pub(crate) fn list_book_shelves_db(
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

pub(crate) fn list_shelf_book_ids_db(
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

pub(crate) fn list_all_shelf_book_ids_db(
    conn: &rusqlite::Connection,
) -> Result<Vec<(i64, i64)>, String> {
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

#[tauri::command]
pub fn create_shelf(state: State<AppState>, name: String) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    create_shelf_db(&db, &name)
}

#[tauri::command]
pub fn list_shelves(state: State<AppState>) -> Result<Vec<Shelf>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_shelves_db(&db)
}

#[tauri::command]
pub fn rename_shelf(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    rename_shelf_db(&db, id, &name)
}

#[tauri::command]
pub fn delete_shelf(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    delete_shelf_db(&db, id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_book_to_shelf(
    state: State<AppState>,
    book_id: i64,
    shelf_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    add_book_to_shelf_db(&db, book_id, shelf_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_book_from_shelf(
    state: State<AppState>,
    book_id: i64,
    shelf_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    remove_book_from_shelf_db(&db, book_id, shelf_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_book_shelves(state: State<AppState>, book_id: i64) -> Result<Vec<Shelf>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_book_shelves_db(&db, book_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_shelf_book_ids(state: State<AppState>, shelf_id: i64) -> Result<Vec<i64>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_shelf_book_ids_db(&db, shelf_id)
}

#[tauri::command]
pub fn list_all_shelf_book_ids(state: State<AppState>) -> Result<Vec<(i64, i64)>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_all_shelf_book_ids_db(&db)
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
