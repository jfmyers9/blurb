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
    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
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

    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    let book = get_book_db(&conn, id).unwrap();
    assert_eq!(book.status, Some("reading".to_string()));

    set_reading_status_db(&conn, id, "finished", None, None).unwrap();
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

    assert!(set_reading_status_db(&conn, id, "bogus", None, None).is_err());
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
    let mut conn = test_conn();
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

    let ids = import_kindle_books_db(&mut conn, &kindle_books).unwrap();
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
        &conn,
        "Shelf Book",
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
        &conn,
        "Removable",
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
        &conn,
        "Cascade Book",
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
fn test_create_shelf_empty_name_errors() {
    let conn = test_conn();
    assert!(create_shelf_db(&conn, "").is_err());
}

#[test]
fn test_create_shelf_whitespace_name_errors() {
    let conn = test_conn();
    assert!(create_shelf_db(&conn, "   ").is_err());
}

#[test]
fn test_rename_shelf_to_duplicate_errors() {
    let conn = test_conn();
    create_shelf_db(&conn, "A").unwrap();
    let id_b = create_shelf_db(&conn, "B").unwrap();
    assert!(rename_shelf_db(&conn, id_b, "A").is_err());
}

#[test]
fn test_rename_nonexistent_shelf_errors() {
    let conn = test_conn();
    assert!(rename_shelf_db(&conn, 99999, "x").is_err());
}

#[test]
fn test_delete_nonexistent_shelf_errors() {
    let conn = test_conn();
    assert!(delete_shelf_db(&conn, 99999).is_err());
}

#[test]
fn test_add_book_to_shelf_idempotent() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn,
        "Idem Book",
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
    let shelf_id = create_shelf_db(&conn, "Idem Shelf").unwrap();

    add_book_to_shelf_db(&conn, book_id, shelf_id).unwrap();
    add_book_to_shelf_db(&conn, book_id, shelf_id).unwrap();

    let ids = list_shelf_book_ids_db(&conn, shelf_id).unwrap();
    assert_eq!(ids.len(), 1);
}

#[test]
fn test_list_all_shelf_book_ids() {
    let conn = test_conn();
    let b1 = add_book_db(
        &conn, "Book 1", None, None, None, None, None, None, None, None,
    )
    .unwrap();
    let b2 = add_book_db(
        &conn, "Book 2", None, None, None, None, None, None, None, None,
    )
    .unwrap();
    let s1 = create_shelf_db(&conn, "Shelf 1").unwrap();
    let s2 = create_shelf_db(&conn, "Shelf 2").unwrap();

    add_book_to_shelf_db(&conn, b1, s1).unwrap();
    add_book_to_shelf_db(&conn, b2, s1).unwrap();
    add_book_to_shelf_db(&conn, b1, s2).unwrap();

    let mut pairs = list_all_shelf_book_ids_db(&conn).unwrap();
    pairs.sort();
    assert_eq!(pairs.len(), 3);
    assert!(pairs.contains(&(s1, b1)));
    assert!(pairs.contains(&(s1, b2)));
    assert!(pairs.contains(&(s2, b1)));
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

fn get_reading_dates(
    conn: &rusqlite::Connection,
    book_id: i64,
) -> (Option<String>, Option<String>) {
    conn.query_row(
        "SELECT started_at, finished_at FROM reading_status WHERE book_id = ?1",
        [book_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .unwrap()
}

#[test]
fn test_reading_status_sets_started_at() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Dates", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "want_to_read", None, None).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(started, None);
    assert_eq!(finished, None);

    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert!(started.is_some(), "reading should set started_at");
    assert_eq!(finished, None);
}

#[test]
fn test_finished_sets_finished_at_preserves_started_at() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Finish", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", Some("2025-01-15"), None).unwrap();
    let (started, _) = get_reading_dates(&conn, id);
    assert_eq!(started, Some("2025-01-15".to_string()));

    set_reading_status_db(&conn, id, "finished", None, None).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(started, Some("2025-01-15".to_string()), "started_at preserved");
    assert!(finished.is_some(), "finished should set finished_at");
}

#[test]
fn test_finished_to_reading_resets_dates() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Reread", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", Some("2025-01-01"), None).unwrap();
    set_reading_status_db(&conn, id, "finished", None, Some("2025-02-01")).unwrap();

    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert!(started.is_some(), "re-reading should set new started_at");
    assert_ne!(started, Some("2025-01-01".to_string()), "started_at should be fresh");
    assert_eq!(finished, None, "finished_at cleared on re-read");
}

#[test]
fn test_want_to_read_clears_both_dates() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Reset", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", Some("2025-03-01"), None).unwrap();
    set_reading_status_db(&conn, id, "finished", None, Some("2025-04-01")).unwrap();

    set_reading_status_db(&conn, id, "want_to_read", None, None).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(started, None, "want_to_read clears started_at");
    assert_eq!(finished, None, "want_to_read clears finished_at");
}

#[test]
fn test_manual_date_override() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Manual", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", Some("2024-06-15"), None).unwrap();
    let (started, _) = get_reading_dates(&conn, id);
    assert_eq!(started, Some("2024-06-15".to_string()));

    set_reading_status_db(&conn, id, "finished", None, Some("2024-07-20")).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(started, Some("2024-06-15".to_string()));
    assert_eq!(finished, Some("2024-07-20".to_string()));
}

#[test]
fn test_invalid_date_format_rejected() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "BadDate", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    assert!(set_reading_status_db(&conn, id, "reading", Some("2024/06/15"), None).is_err());
    assert!(set_reading_status_db(&conn, id, "reading", Some("not-a-date"), None).is_err());
    assert!(set_reading_status_db(&conn, id, "finished", None, Some("06-15-2024")).is_err());
}

#[test]
fn test_import_kindle_sets_started_at() {
    let mut conn = test_conn();
    let kindle_books = vec![KindleBook {
        filename: "test.mobi".to_string(),
        path: "/kindle/test.mobi".to_string(),
        title: "Kindle Date Test".to_string(),
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
    }];

    let ids = import_kindle_books_db(&mut conn, &kindle_books).unwrap();
    let (started, finished) = get_reading_dates(&conn, ids[0]);
    assert!(started.is_some(), "kindle import should set started_at");
    assert_eq!(finished, None);
}
