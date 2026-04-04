use rusqlite::Connection;

pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub up: fn(&Connection) -> Result<(), rusqlite::Error>,
}

fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "initial schema",
            up: |conn| {
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
            },
        },
        Migration {
            version: 2,
            description: "add diary_entries table",
            up: |conn| {
                conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS diary_entries(
                    id INTEGER PRIMARY KEY,
                    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                    body TEXT,
                    rating INTEGER CHECK(rating BETWEEN 1 AND 5),
                    entry_date TEXT NOT NULL DEFAULT (date('now')),
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE INDEX IF NOT EXISTS idx_diary_entries_book_date
                    ON diary_entries(book_id, entry_date DESC, id DESC);",
            )
            },
        },
        Migration {
            version: 3,
            description: "add indexes on foreign key columns",
            up: |conn| {
                conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_reading_status_book_id ON reading_status(book_id);
                 CREATE INDEX IF NOT EXISTS idx_ratings_book_id ON ratings(book_id);
                 CREATE INDEX IF NOT EXISTS idx_reviews_book_id ON reviews(book_id);
                 CREATE INDEX IF NOT EXISTS idx_highlights_book_id ON highlights(book_id);
                 CREATE INDEX IF NOT EXISTS idx_book_shelves_book_id ON book_shelves(book_id);
                 CREATE INDEX IF NOT EXISTS idx_book_shelves_shelf_id ON book_shelves(shelf_id);",
            )
            },
        },
    ]
}

fn get_user_version(conn: &Connection) -> Result<i32, rusqlite::Error> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
}

fn set_user_version(conn: &Connection, version: i32) -> Result<(), rusqlite::Error> {
    conn.execute_batch(&format!("PRAGMA user_version = {version}"))
}

pub fn run_migration_list(conn: &Connection, migrations: &[Migration]) -> Result<(), String> {
    let current_version = get_user_version(conn).map_err(|e| e.to_string())?;
    debug_assert!(migrations.windows(2).all(|w| w[0].version < w[1].version));

    for migration in migrations {
        if migration.version > current_version {
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            (migration.up)(&tx).map_err(|e| {
                format!(
                    "migration {} ({}) failed: {}",
                    migration.version, migration.description, e
                )
            })?;
            set_user_version(&tx, migration.version).map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;

    run_migration_list(conn, &migrations())
}

#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;
