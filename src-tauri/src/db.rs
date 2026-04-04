use rusqlite::Connection;
use std::fs;
use tauri::Manager;

pub fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS books(
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT,
            isbn TEXT,
            asin TEXT,
            cover_url TEXT,
            description TEXT,
            publisher TEXT,
            published_date TEXT,
            page_count INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS reading_status(
            id INTEGER PRIMARY KEY,
            book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            status TEXT NOT NULL CHECK(status IN ('want_to_read','reading','finished','abandoned')),
            started_at TEXT,
            finished_at TEXT,
            updated_at TEXT NOT NULL,
            UNIQUE(book_id)
        );

        CREATE TABLE IF NOT EXISTS ratings(
            id INTEGER PRIMARY KEY,
            book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            score INTEGER NOT NULL CHECK(score BETWEEN 1 AND 5),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(book_id)
        );

        CREATE TABLE IF NOT EXISTS reviews(
            id INTEGER PRIMARY KEY,
            book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(book_id)
        );

        CREATE TABLE IF NOT EXISTS shelves(
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS book_shelves(
            book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            shelf_id INTEGER NOT NULL REFERENCES shelves(id) ON DELETE CASCADE,
            UNIQUE(book_id, shelf_id)
        );

        CREATE TABLE IF NOT EXISTS highlights(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
            text TEXT NOT NULL,
            location_start INTEGER,
            location_end INTEGER,
            page INTEGER,
            clip_type TEXT NOT NULL CHECK(clip_type IN ('highlight','note','bookmark')),
            clipped_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(book_id, text, location_start)
        );",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn init_db(app_handle: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join("books.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    init_schema(&conn)?;

    Ok(conn)
}
