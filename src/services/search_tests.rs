use super::*;
use crate::data::commands::add_book_db;
use crate::data::migrations::run_migrations;

fn test_conn() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    conn
}

#[test]
fn test_search_library_finds_book_by_title() {
    let conn = test_conn();
    add_book_db(
        &conn,
        "The Great Gatsby",
        Some("F. Scott Fitzgerald"),
        None,
        None,
        None,
        Some("A novel about the American dream"),
        None,
        None,
        None,
    )
    .unwrap();

    let results = search_library(&conn, "gatsby");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].book.title, "The Great Gatsby");
}

#[test]
fn test_search_library_finds_book_by_author() {
    let conn = test_conn();
    add_book_db(
        &conn,
        "The Great Gatsby",
        Some("F. Scott Fitzgerald"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    let results = search_library(&conn, "fitzgerald");
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].book.author.as_deref(),
        Some("F. Scott Fitzgerald")
    );
}

#[test]
fn test_search_library_empty_query() {
    let conn = test_conn();
    let results = search_library(&conn, "");
    assert!(results.is_empty());
}

#[test]
fn test_search_library_no_matches() {
    let conn = test_conn();
    add_book_db(
        &conn,
        "The Great Gatsby",
        Some("Fitzgerald"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    let results = search_library(&conn, "nonexistent");
    assert!(results.is_empty());
}

#[test]
fn test_search_library_deduplicates() {
    let conn = test_conn();
    add_book_db(
        &conn,
        "Test Book",
        Some("Test Author"),
        None,
        None,
        None,
        Some("Test description with test"),
        None,
        None,
        None,
    )
    .unwrap();

    let results = search_library(&conn, "test");
    assert_eq!(results.len(), 1);
}
