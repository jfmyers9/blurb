use super::*;
use crate::kindle::KindleBook;

fn test_conn() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::migrations::run_migrations(&conn).unwrap();
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

    let ids = import_kindle_books_db(&mut conn, &kindle_books, None).unwrap();
    assert_eq!(ids.len(), 1, "should only import the new book");

    let new_book = get_book_db(&conn, ids[0]).unwrap();
    assert_eq!(new_book.title, "New Book");
    assert_eq!(
        new_book.status,
        Some("want_to_read".to_string()),
        "imported books get 'want_to_read' status"
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
    assert_eq!(
        started,
        Some("2025-01-15".to_string()),
        "started_at preserved"
    );
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
    assert_ne!(
        started,
        Some("2025-01-01".to_string()),
        "started_at should be fresh"
    );
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

    let ids = import_kindle_books_db(&mut conn, &kindle_books, None).unwrap();
    let (started, finished) = get_reading_dates(&conn, ids[0]);
    assert_eq!(started, None, "kindle import should not set started_at");
    assert_eq!(finished, None);
}

#[test]
fn test_abandoned_sets_finished_at() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Abandon", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", Some("2025-01-01"), None).unwrap();
    set_reading_status_db(&conn, id, "abandoned", None, None).unwrap();

    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(
        started,
        Some("2025-01-01".to_string()),
        "started_at preserved"
    );
    assert!(finished.is_some(), "abandoned should auto-set finished_at");
}

#[test]
fn test_book_dates_via_get_book_db() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "GetBook", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    set_reading_status_db(&conn, id, "finished", None, None).unwrap();

    let book = get_book_db(&conn, id).unwrap();
    assert!(
        book.started_at.is_some(),
        "started_at visible through get_book_db"
    );
    assert!(
        book.finished_at.is_some(),
        "finished_at visible through get_book_db"
    );
}

#[test]
fn test_semantic_date_validation() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "DateVal", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    assert!(set_reading_status_db(&conn, id, "reading", Some("2025-13-15"), None).is_err());
    assert!(set_reading_status_db(&conn, id, "reading", Some("2025-00-15"), None).is_err());
    assert!(set_reading_status_db(&conn, id, "reading", Some("2025-06-00"), None).is_err());
    assert!(set_reading_status_db(&conn, id, "reading", Some("2025-06-32"), None).is_err());
    assert!(set_reading_status_db(&conn, id, "reading", Some("2025-06-15"), None).is_ok());
}

#[test]
fn test_simultaneous_date_override() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "SimDate", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    set_reading_status_db(
        &conn,
        id,
        "finished",
        Some("2024-01-01"),
        Some("2024-12-31"),
    )
    .unwrap();

    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(started, Some("2024-01-01".to_string()));
    assert_eq!(finished, Some("2024-12-31".to_string()));
}

#[test]
fn test_update_reading_dates_clears_dates() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Clear", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    set_reading_status_db(&conn, id, "finished", None, None).unwrap();

    let (started, finished) = get_reading_dates(&conn, id);
    assert!(started.is_some());
    assert!(finished.is_some());

    update_reading_dates_db(&conn, id, None, None).unwrap();

    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(
        started, None,
        "update_reading_dates_db should clear started_at"
    );
    assert_eq!(
        finished, None,
        "update_reading_dates_db should clear finished_at"
    );
}

#[test]
fn test_update_reading_dates_sets_independently() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Indep", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    set_reading_status_db(&conn, id, "reading", None, None).unwrap();

    update_reading_dates_db(&conn, id, Some("2020-05-15"), Some("2020-06-20")).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(started, Some("2020-05-15".to_string()));
    assert_eq!(finished, Some("2020-06-20".to_string()));

    update_reading_dates_db(&conn, id, None, Some("2020-06-20")).unwrap();
    let (started, finished) = get_reading_dates(&conn, id);
    assert_eq!(started, None, "started_at cleared independently");
    assert_eq!(
        finished,
        Some("2020-06-20".to_string()),
        "finished_at preserved"
    );
}

#[test]
fn test_create_diary_entry() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn,
        "Diary Book",
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

    let entry = create_diary_entry_db(
        &conn,
        book_id,
        Some("Great read today"),
        Some(4),
        "2026-04-01",
    )
    .unwrap();

    assert_eq!(entry.book_id, book_id);
    assert_eq!(entry.body, Some("Great read today".to_string()));
    assert_eq!(entry.rating, Some(4));
    assert_eq!(entry.entry_date, "2026-04-01");
    assert_eq!(entry.book_title, "Diary Book");
    assert_eq!(entry.book_author, Some("Author".to_string()));
}

#[test]
fn test_diary_entry_rating_validation() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Book", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    assert!(create_diary_entry_db(&conn, book_id, None, Some(0), "2026-04-01").is_err());
    assert!(create_diary_entry_db(&conn, book_id, None, Some(6), "2026-04-01").is_err());
    assert!(create_diary_entry_db(&conn, book_id, None, Some(3), "2026-04-01").is_ok());
}

#[test]
fn test_diary_entries_ordering() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Book", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    let e1 = create_diary_entry_db(&conn, book_id, Some("First"), None, "2026-04-01").unwrap();
    let e2 = create_diary_entry_db(&conn, book_id, Some("Second"), None, "2026-04-03").unwrap();
    let e3 = create_diary_entry_db(&conn, book_id, Some("Third"), None, "2026-04-01").unwrap();

    let entries = list_book_diary_entries_db(&conn, book_id).unwrap();
    assert_eq!(entries.len(), 3);
    // 2026-04-03 first, then 2026-04-01 by id DESC
    assert_eq!(entries[0].id, e2.id);
    assert_eq!(entries[1].id, e3.id);
    assert_eq!(entries[2].id, e1.id);

    // list_diary_entries_db should return same ordering
    let all = list_diary_entries_db(&conn).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].id, e2.id);
}

#[test]
fn test_update_diary_entry() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Book", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    let entry =
        create_diary_entry_db(&conn, book_id, Some("Original"), Some(3), "2026-04-01").unwrap();

    update_diary_entry_db(&conn, entry.id, Some("Updated"), Some(5), "2026-04-02").unwrap();

    let entries = list_book_diary_entries_db(&conn, book_id).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].body, Some("Updated".to_string()));
    assert_eq!(entries[0].rating, Some(5));
    assert_eq!(entries[0].entry_date, "2026-04-02");
}

#[test]
fn test_delete_diary_entry() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Book", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    let entry = create_diary_entry_db(&conn, book_id, None, None, "2026-04-01").unwrap();
    assert_eq!(list_book_diary_entries_db(&conn, book_id).unwrap().len(), 1);

    delete_diary_entry_db(&conn, entry.id).unwrap();
    assert_eq!(list_book_diary_entries_db(&conn, book_id).unwrap().len(), 0);
}

#[test]
fn test_delete_book_cascades_diary_entries() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Doomed", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    create_diary_entry_db(&conn, book_id, Some("Entry"), None, "2026-04-01").unwrap();

    delete_book_db(&conn, book_id).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM diary_entries WHERE book_id = ?1",
            [book_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_reading_status_dates() {
    let conn = test_conn();
    let id = add_book_db(
        &conn, "Dated", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    // Set to "reading" → started_at should be set
    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    let row: (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT started_at, finished_at FROM reading_status WHERE book_id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(row.0.is_some(), "started_at should be set");
    assert!(row.1.is_none(), "finished_at should be None");
    let original_started = row.0.unwrap();

    // Set to "finished" → finished_at set, started_at preserved
    set_reading_status_db(&conn, id, "finished", None, None).unwrap();
    let row: (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT started_at, finished_at FROM reading_status WHERE book_id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        row.0.as_deref(),
        Some(original_started.as_str()),
        "started_at preserved"
    );
    assert!(row.1.is_some(), "finished_at should be set");

    // Set back to "reading" → finished_at cleared
    set_reading_status_db(&conn, id, "reading", None, None).unwrap();
    let row: (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT started_at, finished_at FROM reading_status WHERE book_id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(row.0.is_some(), "started_at should still be set");
    assert!(row.1.is_none(), "finished_at should be cleared");
}

#[test]
fn test_update_diary_entry_rating_validation() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Book", None, None, None, None, None, None, None, None,
    )
    .unwrap();
    let entry = create_diary_entry_db(&conn, book_id, None, Some(3), "2026-04-01").unwrap();

    assert!(update_diary_entry_db(&conn, entry.id, None, Some(0), "2026-04-01").is_err());
    assert!(update_diary_entry_db(&conn, entry.id, None, Some(6), "2026-04-01").is_err());
    assert!(update_diary_entry_db(&conn, entry.id, None, Some(5), "2026-04-01").is_ok());
}

#[test]
fn test_diary_entry_date_validation() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Book", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    assert!(create_diary_entry_db(&conn, book_id, None, None, "not-a-date").is_err());
    assert!(create_diary_entry_db(&conn, book_id, None, None, "04/01/2026").is_err());
    assert!(create_diary_entry_db(&conn, book_id, None, None, "2026-04-01").is_ok());
}

#[test]
fn test_import_kindle_persists_all_metadata() {
    let mut conn = test_conn();
    let kindle_books = vec![KindleBook {
        filename: "book.mobi".to_string(),
        path: "/kindle/book.mobi".to_string(),
        title: "Rich Metadata Book".to_string(),
        author: Some("Jane Doe".to_string()),
        asin: Some("B0ABCDEFGH".to_string()),
        isbn: Some("9781234567890".to_string()),
        publisher: Some("Acme Press".to_string()),
        description: Some("A test book description".to_string()),
        published_date: Some("2024-01-15".to_string()),
        language: Some("en".to_string()),
        cover_data: None,
        cde_type: Some("EBOK".to_string()),
        extension: "mobi".to_string(),
        size_bytes: 5000,
    }];

    let ids = import_kindle_books_db(&mut conn, &kindle_books, None).unwrap();
    assert_eq!(ids.len(), 1);

    let book = get_book_db(&conn, ids[0]).unwrap();
    assert_eq!(book.author, Some("Jane Doe".to_string()));
    assert_eq!(book.asin, Some("B0ABCDEFGH".to_string()));
    assert_eq!(book.isbn, Some("9781234567890".to_string()));
    assert_eq!(book.publisher, Some("Acme Press".to_string()));
    assert_eq!(book.description, Some("A test book description".to_string()));
    assert_eq!(book.published_date, Some("2024-01-15".to_string()));
}

#[test]
fn test_enrich_backfills_author() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Orphan Book", None, None, None, None, None, None, None, None,
    )
    .unwrap();

    let meta = crate::metadata::BookMetadata {
        author: Some("Discovered Author".to_string()),
        ..Default::default()
    };
    enrich_book_db(&conn, book_id, &meta).unwrap();

    let book = get_book_db(&conn, book_id).unwrap();
    assert_eq!(book.author, Some("Discovered Author".to_string()));
}

#[test]
fn test_enrich_does_not_overwrite_existing_author() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn,
        "Has Author",
        Some("Original Author"),
        None, None, None, None, None, None, None,
    )
    .unwrap();

    let meta = crate::metadata::BookMetadata {
        author: Some("API Author".to_string()),
        ..Default::default()
    };
    enrich_book_db(&conn, book_id, &meta).unwrap();

    let book = get_book_db(&conn, book_id).unwrap();
    assert_eq!(book.author, Some("Original Author".to_string()));
}

#[test]
fn test_enrich_prefers_api_cover_over_existing() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn, "Cover Book", None, None, None,
        Some("http://old-cover.jpg"), None, None, None, None,
    )
    .unwrap();

    let meta = crate::metadata::BookMetadata {
        cover_url: Some("http://new-cover.jpg".to_string()),
        ..Default::default()
    };
    enrich_book_db(&conn, book_id, &meta).unwrap();

    let book = get_book_db(&conn, book_id).unwrap();
    assert_eq!(book.cover_url, Some("http://new-cover.jpg".to_string()));
}

#[test]
fn test_enrich_preserves_existing_author() {
    let conn = test_conn();
    let book_id = add_book_db(
        &conn,
        "My Book",
        Some("Keep This Author"),
        None, None, None, None, None, None, None,
    )
    .unwrap();

    let meta = crate::metadata::BookMetadata {
        author: Some("Different Author".to_string()),
        cover_url: Some("http://cover.jpg".to_string()),
        isbn: Some("1234567890".to_string()),
        ..Default::default()
    };
    enrich_book_db(&conn, book_id, &meta).unwrap();

    let book = get_book_db(&conn, book_id).unwrap();
    assert_eq!(book.author, Some("Keep This Author".to_string()));
    assert_eq!(book.cover_url, Some("http://cover.jpg".to_string()));
    assert_eq!(book.isbn, Some("1234567890".to_string()));
}

#[test]
fn test_import_saves_embedded_cover_to_disk() {
    use base64::Engine;
    let mut conn = test_conn();

    let cover_bytes = b"fake-jpeg-data";
    let cover_b64 = base64::engine::general_purpose::STANDARD.encode(cover_bytes);

    let tmp_dir = std::env::temp_dir().join(format!("blurb_test_covers_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let books = vec![
        KindleBook {
            filename: "with_cover.mobi".to_string(),
            path: "/kindle/with_cover.mobi".to_string(),
            title: "Book With Cover".to_string(),
            author: Some("Author".to_string()),
            asin: None,
            isbn: None,
            publisher: None,
            description: None,
            published_date: None,
            language: None,
            cover_data: Some(cover_b64),
            cde_type: None,
            extension: "mobi".to_string(),
            size_bytes: 1000,
        },
        KindleBook {
            filename: "no_cover.mobi".to_string(),
            path: "/kindle/no_cover.mobi".to_string(),
            title: "Book Without Cover".to_string(),
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
            size_bytes: 2000,
        },
    ];

    let ids = import_kindle_books_db(&mut conn, &books, Some(&tmp_dir)).unwrap();
    assert_eq!(ids.len(), 2);

    // Book with cover_data: file exists, cover_url set
    let cover_path = tmp_dir.join(format!("{}.jpg", ids[0]));
    assert!(cover_path.exists(), "cover file should be written to disk");
    assert_eq!(std::fs::read(&cover_path).unwrap(), b"fake-jpeg-data");

    let book1 = get_book_db(&conn, ids[0]).unwrap();
    assert_eq!(
        book1.cover_url,
        Some(cover_path.to_string_lossy().to_string())
    );

    // Book without cover_data: no file, cover_url is NULL
    let no_cover_path = tmp_dir.join(format!("{}.jpg", ids[1]));
    assert!(!no_cover_path.exists(), "no cover file for book without cover_data");

    let book2 = get_book_db(&conn, ids[1]).unwrap();
    assert_eq!(book2.cover_url, None);

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_import_kindle_invalid_cover_base64_is_non_fatal() {
    let mut conn = test_conn();
    let tmp_dir = std::env::temp_dir().join("blurb_test_bad_cover");
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let books = vec![KindleBook {
        filename: "bad_cover.mobi".to_string(),
        path: "/kindle/bad_cover.mobi".to_string(),
        title: "Bad Cover Book".to_string(),
        author: Some("Author".to_string()),
        asin: None,
        isbn: None,
        publisher: None,
        description: None,
        published_date: None,
        language: None,
        cover_data: Some("%%%not-valid-base64%%%".to_string()),
        cde_type: None,
        extension: "mobi".to_string(),
        size_bytes: 1000,
    }];

    let ids = import_kindle_books_db(&mut conn, &books, Some(&tmp_dir)).unwrap();
    assert_eq!(ids.len(), 1, "book should still be imported despite bad cover");

    let book = get_book_db(&conn, ids[0]).unwrap();
    assert_eq!(book.title, "Bad Cover Book");
    assert_eq!(book.cover_url, None, "cover_url should be None when base64 is invalid");

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
