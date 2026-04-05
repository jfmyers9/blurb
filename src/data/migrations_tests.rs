use super::*;
use rusqlite::Connection;

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
        vec![
            "book_notes",
            "book_shelves",
            "books",
            "books_fts",
            "books_fts_config",
            "books_fts_data",
            "books_fts_docsize",
            "books_fts_idx",
            "collection_books",
            "collections",
            "diary_entries",
            "highlights",
            "ratings",
            "reading_goals",
            "reading_status",
            "reviews",
            "settings",
            "shelves"
        ]
    );
}

#[test]
fn run_migrations_preserves_existing_version() {
    let conn = Connection::open_in_memory().unwrap();
    set_user_version(&conn, 99).unwrap();
    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 99);
}

#[test]
fn fresh_db_reaches_latest_version_with_indexes() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 8);

    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name",
        )
        .unwrap();
    let indexes: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        indexes,
        vec![
            "idx_book_notes_book_id",
            "idx_book_shelves_book_id",
            "idx_book_shelves_shelf_id",
            "idx_collection_books_collection",
            "idx_diary_entries_book_date",
            "idx_highlights_book_id",
            "idx_ratings_book_id",
            "idx_reading_status_book_id",
            "idx_reviews_book_id",
        ]
    );
}

#[test]
fn incremental_upgrade_from_version_1_to_latest() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

    run_migration_list(&conn, &migrations()[..1]).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 1);

    conn.execute(
        "INSERT INTO books (id, title, created_at, updated_at) VALUES (1, 'Test Book', datetime('now'), datetime('now'))",
        [],
    )
    .unwrap();

    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 8);

    let title: String = conn
        .query_row("SELECT title FROM books WHERE id = 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(title, "Test Book");

    let idx_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(idx_count, 9);
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
    assert!(
        err.contains("migration 1"),
        "error should contain version number: {err}"
    );
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
    assert!(
        err.contains("migration 2"),
        "error should reference version 2: {err}"
    );
    assert_eq!(
        get_user_version(&conn).unwrap(),
        1,
        "user_version should stay at 1 after second migration fails"
    );
    conn.execute("INSERT INTO test_tbl (id) VALUES (1)", [])
        .expect("test_tbl should exist from successful first migration");
}

#[test]
fn pragma_user_version_rolls_back_with_transaction() {
    let conn = Connection::open_in_memory().unwrap();
    set_user_version(&conn, 1).unwrap();
    let tx = conn.unchecked_transaction().unwrap();
    set_user_version(&tx, 42).unwrap();
    tx.rollback().unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 1);
}

#[test]
fn run_migrations_is_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    run_migrations(&conn).unwrap();
    assert_eq!(get_user_version(&conn).unwrap(), 8);
}

#[test]
fn migration_versions_are_strictly_ascending() {
    let ms = migrations();
    for w in ms.windows(2) {
        assert!(
            w[0].version < w[1].version,
            "migration {} must come before {}",
            w[0].version,
            w[1].version
        );
    }
}

#[test]
fn run_migrations_enables_foreign_keys() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    let fk: i32 = conn
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1);
}
