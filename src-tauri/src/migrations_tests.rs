use super::*;
use rusqlite::Connection;

#[test]
fn run_migrations_on_fresh_db() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 2);
}

#[test]
fn migration_1_creates_all_tables() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        tables,
        vec!["book_shelves", "books", "highlights", "ratings", "reading_status", "reviews", "shelves"]
    );
}

#[test]
fn run_migrations_preserves_existing_version() {
    let conn = Connection::open_in_memory().unwrap();
    set_user_version(&conn, 5).unwrap();
    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 5);
}

#[test]
fn fresh_db_reaches_version_2_with_indexes() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 2);

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name")
        .unwrap();
    let indexes: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        indexes,
        vec![
            "idx_book_shelves_book_id",
            "idx_book_shelves_shelf_id",
            "idx_highlights_book_id",
            "idx_ratings_book_id",
            "idx_reading_status_book_id",
            "idx_reviews_book_id",
        ]
    );
}

#[test]
fn incremental_upgrade_from_version_1_to_2() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

    // Apply only migration 1
    let v1_only = vec![Migration {
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
    }];
    run_migration_list(&conn, &v1_only).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 1);

    // Insert data at version 1
    conn.execute(
        "INSERT INTO books (id, title, created_at, updated_at) VALUES (1, 'Test Book', datetime('now'), datetime('now'))",
        [],
    )
    .unwrap();

    // Run full migrations — should only apply migration 2
    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 2);

    // Data from version 1 is preserved
    let title: String = conn
        .query_row("SELECT title FROM books WHERE id = 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(title, "Test Book");

    // Indexes exist
    let idx_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(idx_count, 6);
}

#[test]
fn failed_migration_includes_version_and_sql_error() {
    let conn = Connection::open_in_memory().unwrap();
    let migrations = vec![Migration {
        version: 1,
        description: "broken schema",
        up: |conn| conn.execute_batch("CREATE TABL invalid_syntax"),
    }];

    let err = run_migration_list(&conn, &migrations).unwrap_err();
    assert!(err.contains("1"), "error should contain version number: {err}");
    assert!(
        err.contains("broken schema"),
        "error should contain description: {err}"
    );
    assert!(
        err.contains("syntax error") || err.contains("near"),
        "error should contain SQL error detail: {err}"
    );
}

#[test]
fn failed_migration_does_not_advance_user_version() {
    let conn = Connection::open_in_memory().unwrap();
    let migrations = vec![
        Migration {
            version: 1,
            description: "create test table",
            up: |conn| conn.execute_batch("CREATE TABLE test_tbl (id INTEGER PRIMARY KEY)"),
        },
        Migration {
            version: 2,
            description: "broken migration",
            up: |conn| conn.execute_batch("CREATE TABL invalid_syntax"),
        },
    ];

    let err = run_migration_list(&conn, &migrations).unwrap_err();
    assert!(err.contains("2"), "error should reference version 2: {err}");
    assert_eq!(
        get_user_version(&conn).unwrap(),
        1,
        "user_version should stay at 1 after second migration fails"
    );
    conn.execute("INSERT INTO test_tbl (id) VALUES (1)", [])
        .expect("test_tbl should exist from successful first migration");
}
