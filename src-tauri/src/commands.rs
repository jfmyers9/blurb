use crate::kindle::KindleBook;
use crate::metadata::BookMetadata;
use crate::models::Book;
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
        rusqlite::params![title, author, isbn, asin, cover_url, description, publisher, published_date, page_count],
    )?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_books_db(conn: &rusqlite::Connection) -> Result<Vec<Book>, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY b.updated_at DESC", BOOK_SELECT))?;
    let books = stmt
        .query_map([], |row| row_to_book(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(books)
}

pub(crate) fn get_book_db(conn: &rusqlite::Connection, id: i64) -> Result<Book, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("{} WHERE b.id = ?1", BOOK_SELECT))?;
    stmt.query_row([id], |row| row_to_book(row))
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
        rusqlite::params![title, author, isbn, asin, cover_url, description, publisher, published_date, page_count, id],
    )?;
    get_book_db(conn, id)
}

pub(crate) fn delete_book_db(conn: &rusqlite::Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM books WHERE id = ?1", [id])?;
    Ok(())
}

pub(crate) fn set_rating_db(conn: &rusqlite::Connection, book_id: i64, score: i32) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO ratings (book_id, score, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET score=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, score],
    )?;
    Ok(())
}

pub(crate) fn set_reading_status_db(conn: &rusqlite::Connection, book_id: i64, status: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO reading_status (book_id, status, updated_at) \
         VALUES (?1, ?2, datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET status=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, status],
    )?;
    Ok(())
}

pub(crate) fn save_review_db(conn: &rusqlite::Connection, book_id: i64, body: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO reviews (book_id, body, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET body=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, body],
    )?;
    Ok(())
}

pub(crate) fn import_kindle_books_db(conn: &rusqlite::Connection, books: &[KindleBook]) -> Result<Vec<i64>, rusqlite::Error> {
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
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");

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
        let id = add_book_db(&conn, "Test Book", Some("Test Author"), None, None, None, None, None, None, None).unwrap();

        let books = list_books_db(&conn).unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, id);
        assert_eq!(books[0].title, "Test Book");
        assert_eq!(books[0].author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_get_book() {
        let conn = test_conn();
        let id = add_book_db(&conn, "Get Me", Some("Author"), None, None, None, None, None, None, None).unwrap();

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
        let id = add_book_db(&conn, "Old Title", Some("Old Author"), None, None, None, None, None, None, None).unwrap();

        let book = update_book_db(&conn, id, "New Title", Some("New Author"), None, None, None, None, None, None, None).unwrap();
        assert_eq!(book.title, "New Title");
        assert_eq!(book.author, Some("New Author".to_string()));
    }

    #[test]
    fn test_delete_book_cascades() {
        let conn = test_conn();
        let id = add_book_db(&conn, "Doomed", None, None, None, None, None, None, None, None).unwrap();

        set_rating_db(&conn, id, 4).unwrap();
        set_reading_status_db(&conn, id, "reading").unwrap();
        save_review_db(&conn, id, "Great").unwrap();

        delete_book_db(&conn, id).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM ratings WHERE book_id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM reading_status WHERE book_id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM reviews WHERE book_id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_set_rating() {
        let conn = test_conn();
        let id = add_book_db(&conn, "Rated", None, None, None, None, None, None, None, None).unwrap();

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
        let id = add_book_db(&conn, "Bad Rating", None, None, None, None, None, None, None, None).unwrap();

        assert!(set_rating_db(&conn, id, 0).is_err());
        assert!(set_rating_db(&conn, id, 6).is_err());
    }

    #[test]
    fn test_set_reading_status() {
        let conn = test_conn();
        let id = add_book_db(&conn, "Status Book", None, None, None, None, None, None, None, None).unwrap();

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
        let id = add_book_db(&conn, "Invalid Status", None, None, None, None, None, None, None, None).unwrap();

        assert!(set_reading_status_db(&conn, id, "bogus").is_err());
    }

    #[test]
    fn test_save_review() {
        let conn = test_conn();
        let id = add_book_db(&conn, "Reviewed", None, None, None, None, None, None, None, None).unwrap();

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
        add_book_db(&conn, "Existing Book", Some("Author"), None, None, None, None, None, None, None).unwrap();

        let kindle_books = vec![
            KindleBook {
                filename: "existing.mobi".to_string(),
                path: "/kindle/existing.mobi".to_string(),
                title: "Existing Book".to_string(),
                author: Some("Author".to_string()),
                extension: "mobi".to_string(),
                size_bytes: 1000,
            },
            KindleBook {
                filename: "new.mobi".to_string(),
                path: "/kindle/new.mobi".to_string(),
                title: "New Book".to_string(),
                author: Some("New Author".to_string()),
                extension: "mobi".to_string(),
                size_bytes: 2000,
            },
        ];

        let ids = import_kindle_books_db(&conn, &kindle_books).unwrap();
        assert_eq!(ids.len(), 1, "should only import the new book");

        let new_book = get_book_db(&conn, ids[0]).unwrap();
        assert_eq!(new_book.title, "New Book");
        assert_eq!(new_book.status, Some("reading".to_string()), "imported books get 'reading' status");

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2);
    }
}
