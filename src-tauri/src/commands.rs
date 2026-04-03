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
    db.execute(
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
    Ok(db.last_insert_rowid())
}

#[tauri::command]
pub fn list_books(state: State<AppState>) -> Result<Vec<Book>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare(&format!("{} ORDER BY b.updated_at DESC", BOOK_SELECT))
        .map_err(|e| e.to_string())?;
    let books = stmt
        .query_map([], |row| row_to_book(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(books)
}

#[tauri::command]
pub fn get_book(state: State<AppState>, id: i64) -> Result<Book, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare(&format!("{} WHERE b.id = ?1", BOOK_SELECT))
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], |row| row_to_book(row))
        .map_err(|e| e.to_string())
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
    db.execute(
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

    let mut stmt = db
        .prepare(&format!("{} WHERE b.id = ?1", BOOK_SELECT))
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], |row| row_to_book(row))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_book(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM books WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

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
    db.execute(
        "INSERT INTO ratings (book_id, score, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET score=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, score],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_reading_status(
    state: State<AppState>,
    book_id: i64,
    status: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO reading_status (book_id, status, updated_at) \
         VALUES (?1, ?2, datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET status=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, status],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_review(state: State<AppState>, book_id: i64, body: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO reviews (book_id, body, created_at, updated_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now')) \
         ON CONFLICT(book_id) DO UPDATE SET body=?2, updated_at=datetime('now')",
        rusqlite::params![book_id, body],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
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
    let mut ids = Vec::new();

    for book in &books {
        let exists: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM books WHERE title = ?1)",
                [&book.title],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if exists {
            continue;
        }

        db.execute(
            "INSERT INTO books (title, author, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            rusqlite::params![book.title, book.author],
        )
        .map_err(|e| e.to_string())?;

        let book_id = db.last_insert_rowid();

        db.execute(
            "INSERT INTO reading_status (book_id, status, updated_at) \
             VALUES (?1, 'reading', datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET status='reading', updated_at=datetime('now')",
            [book_id],
        )
        .map_err(|e| e.to_string())?;

        ids.push(book_id);
    }

    Ok(ids)
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

    fn test_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::init_schema(&conn).unwrap();
        conn
    }

    fn insert_book(conn: &rusqlite::Connection, title: &str, author: Option<&str>) -> i64 {
        conn.execute(
            "INSERT INTO books (title, author, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            rusqlite::params![title, author],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn get_book_by_id(conn: &rusqlite::Connection, id: i64) -> Book {
        let mut stmt = conn
            .prepare(&format!("{} WHERE b.id = ?1", BOOK_SELECT))
            .unwrap();
        stmt.query_row([id], |row| row_to_book(row)).unwrap()
    }

    #[test]
    fn test_add_and_list_books() {
        let conn = test_conn();
        let id = insert_book(&conn, "Test Book", Some("Test Author"));

        let mut stmt = conn
            .prepare(&format!("{} ORDER BY b.updated_at DESC", BOOK_SELECT))
            .unwrap();
        let books: Vec<Book> = stmt
            .query_map([], |row| row_to_book(row))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, id);
        assert_eq!(books[0].title, "Test Book");
        assert_eq!(books[0].author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_get_book() {
        let conn = test_conn();
        let id = insert_book(&conn, "Get Me", Some("Author"));
        let book = get_book_by_id(&conn, id);
        assert_eq!(book.title, "Get Me");
        assert_eq!(book.author, Some("Author".to_string()));
        assert_eq!(book.rating, None);
        assert_eq!(book.status, None);
        assert_eq!(book.review, None);
    }

    #[test]
    fn test_update_book() {
        let conn = test_conn();
        let id = insert_book(&conn, "Old Title", Some("Old Author"));

        conn.execute(
            "UPDATE books SET title=?1, author=?2, updated_at=datetime('now') WHERE id=?3",
            rusqlite::params!["New Title", "New Author", id],
        )
        .unwrap();

        let book = get_book_by_id(&conn, id);
        assert_eq!(book.title, "New Title");
        assert_eq!(book.author, Some("New Author".to_string()));
    }

    #[test]
    fn test_delete_book_cascades() {
        let conn = test_conn();
        let id = insert_book(&conn, "Doomed", None);

        conn.execute(
            "INSERT INTO ratings (book_id, score, created_at, updated_at) \
             VALUES (?1, 4, datetime('now'), datetime('now'))",
            [id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO reading_status (book_id, status, updated_at) \
             VALUES (?1, 'reading', datetime('now'))",
            [id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO reviews (book_id, body, created_at, updated_at) \
             VALUES (?1, 'Great', datetime('now'), datetime('now'))",
            [id],
        )
        .unwrap();

        conn.execute("DELETE FROM books WHERE id = ?1", [id])
            .unwrap();

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
        let id = insert_book(&conn, "Rated", None);

        conn.execute(
            "INSERT INTO ratings (book_id, score, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET score=?2, updated_at=datetime('now')",
            rusqlite::params![id, 3],
        )
        .unwrap();

        let book = get_book_by_id(&conn, id);
        assert_eq!(book.rating, Some(3));

        conn.execute(
            "INSERT INTO ratings (book_id, score, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET score=?2, updated_at=datetime('now')",
            rusqlite::params![id, 5],
        )
        .unwrap();

        let book = get_book_by_id(&conn, id);
        assert_eq!(book.rating, Some(5));
    }

    #[test]
    fn test_rating_constraint() {
        let conn = test_conn();
        let id = insert_book(&conn, "Bad Rating", None);

        let result = conn.execute(
            "INSERT INTO ratings (book_id, score, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            rusqlite::params![id, 0],
        );
        assert!(result.is_err());

        let result = conn.execute(
            "INSERT INTO ratings (book_id, score, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now'))",
            rusqlite::params![id, 6],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_set_reading_status() {
        let conn = test_conn();
        let id = insert_book(&conn, "Status Book", None);

        conn.execute(
            "INSERT INTO reading_status (book_id, status, updated_at) \
             VALUES (?1, ?2, datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET status=?2, updated_at=datetime('now')",
            rusqlite::params![id, "reading"],
        )
        .unwrap();

        let book = get_book_by_id(&conn, id);
        assert_eq!(book.status, Some("reading".to_string()));

        conn.execute(
            "INSERT INTO reading_status (book_id, status, updated_at) \
             VALUES (?1, ?2, datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET status=?2, updated_at=datetime('now')",
            rusqlite::params![id, "finished"],
        )
        .unwrap();

        let book = get_book_by_id(&conn, id);
        assert_eq!(book.status, Some("finished".to_string()));
    }

    #[test]
    fn test_invalid_reading_status() {
        let conn = test_conn();
        let id = insert_book(&conn, "Invalid Status", None);

        let result = conn.execute(
            "INSERT INTO reading_status (book_id, status, updated_at) \
             VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![id, "bogus"],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_save_review() {
        let conn = test_conn();
        let id = insert_book(&conn, "Reviewed", None);

        conn.execute(
            "INSERT INTO reviews (book_id, body, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET body=?2, updated_at=datetime('now')",
            rusqlite::params![id, "Amazing book"],
        )
        .unwrap();

        let book = get_book_by_id(&conn, id);
        assert_eq!(book.review, Some("Amazing book".to_string()));

        conn.execute(
            "INSERT INTO reviews (book_id, body, created_at, updated_at) \
             VALUES (?1, ?2, datetime('now'), datetime('now')) \
             ON CONFLICT(book_id) DO UPDATE SET body=?2, updated_at=datetime('now')",
            rusqlite::params![id, "Updated review"],
        )
        .unwrap();

        let book = get_book_by_id(&conn, id);
        assert_eq!(book.review, Some("Updated review".to_string()));
    }

    #[test]
    fn test_import_kindle_skips_duplicates() {
        let conn = test_conn();
        insert_book(&conn, "Existing Book", Some("Author"));

        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM books WHERE title = ?1)",
                ["Existing Book"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(exists);

        let count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))
            .unwrap();

        let title = "Existing Book";
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM books WHERE title = ?1)",
                [title],
                |row| row.get(0),
            )
            .unwrap();
        if !exists {
            insert_book(&conn, title, None);
        }

        let count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count_before, count_after);
    }
}
