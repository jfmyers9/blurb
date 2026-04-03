use crate::kindle::KindleBook;
use crate::metadata::BookMetadata;
use crate::models::Book;
use crate::AppState;
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
pub fn delete_book(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM books WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
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
