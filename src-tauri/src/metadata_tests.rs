use super::*;

#[test]
fn sanitize_isbn_with_dashes() {
    assert_eq!(sanitize_isbn("978-0-14-143951-8"), "9780141439518");
}

#[test]
fn sanitize_isbn_with_spaces() {
    assert_eq!(sanitize_isbn("978 0141439518"), "9780141439518");
}

#[test]
fn sanitize_isbn_empty() {
    assert_eq!(sanitize_isbn(""), "");
}

#[test]
fn sanitize_isbn_already_clean() {
    assert_eq!(sanitize_isbn("9780141439518"), "9780141439518");
}

#[test]
fn prefer_isbn13_picks_978() {
    let isbns = Some(vec![
        "0141439518".into(),
        "9780141439518".into(),
        "1234567890".into(),
    ]);
    assert_eq!(prefer_isbn13(isbns), Some("9780141439518".into()));
}

#[test]
fn prefer_isbn13_picks_979() {
    let isbns = Some(vec!["0141439518".into(), "9791234567890".into()]);
    assert_eq!(prefer_isbn13(isbns), Some("9791234567890".into()));
}

#[test]
fn prefer_isbn13_falls_back_to_first() {
    let isbns = Some(vec!["0141439518".into(), "1234567890".into()]);
    assert_eq!(prefer_isbn13(isbns), Some("0141439518".into()));
}

#[test]
fn prefer_isbn13_empty_list() {
    assert_eq!(prefer_isbn13(Some(vec![])), None);
}

#[test]
fn prefer_isbn13_none() {
    assert_eq!(prefer_isbn13(None), None);
}

#[test]
fn dedup_removes_duplicate_keys() {
    let docs = vec![
        OLSearchDoc {
            title: Some("A".into()),
            key: Some("/works/OL1".into()),
            author_name: None,
            cover_i: None,
            isbn: None,
            publisher: None,
            first_publish_year: None,
            number_of_pages_median: None,
        },
        OLSearchDoc {
            title: Some("A dup".into()),
            key: Some("/works/OL1".into()),
            author_name: None,
            cover_i: None,
            isbn: None,
            publisher: None,
            first_publish_year: None,
            number_of_pages_median: None,
        },
        OLSearchDoc {
            title: Some("B".into()),
            key: Some("/works/OL2".into()),
            author_name: None,
            cover_i: None,
            isbn: None,
            publisher: None,
            first_publish_year: None,
            number_of_pages_median: None,
        },
    ];
    let mut seen = std::collections::HashSet::new();
    let results: Vec<_> = docs
        .into_iter()
        .filter(|doc| doc.key.as_ref().is_none_or(|k| seen.insert(k.clone())))
        .collect();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title.as_deref(), Some("A"));
    assert_eq!(results[1].title.as_deref(), Some("B"));
}

#[test]
fn dedup_keyless_docs_pass_through() {
    let docs = vec![
        OLSearchDoc {
            title: Some("No key 1".into()),
            key: None,
            author_name: None,
            cover_i: None,
            isbn: None,
            publisher: None,
            first_publish_year: None,
            number_of_pages_median: None,
        },
        OLSearchDoc {
            title: Some("No key 2".into()),
            key: None,
            author_name: None,
            cover_i: None,
            isbn: None,
            publisher: None,
            first_publish_year: None,
            number_of_pages_median: None,
        },
    ];
    let mut seen = std::collections::HashSet::new();
    let results: Vec<_> = docs
        .into_iter()
        .filter(|doc| doc.key.as_ref().is_none_or(|k| seen.insert(k.clone())))
        .collect();
    assert_eq!(results.len(), 2);
}

#[test]
fn dedup_empty_vec() {
    let docs: Vec<OLSearchDoc> = vec![];
    let mut seen = std::collections::HashSet::new();
    let results: Vec<_> = docs
        .into_iter()
        .filter(|doc| doc.key.as_ref().is_none_or(|k| seen.insert(k.clone())))
        .collect();
    assert!(results.is_empty());
}

#[test]
fn ol_search_doc_to_book_metadata() {
    let doc = OLSearchDoc {
        title: Some("Great Expectations".into()),
        author_name: Some(vec!["Charles Dickens".into()]),
        cover_i: Some(12345),
        isbn: Some(vec!["0141439564".into(), "9780141439563".into()]),
        key: Some("/works/OL45804W".into()),
        publisher: Some(vec!["Penguin".into()]),
        first_publish_year: Some(1861),
        number_of_pages_median: Some(544),
    };

    let meta = BookMetadata {
        title: doc.title,
        author: doc.author_name.as_ref().and_then(|a| a.first().cloned()),
        cover_url: doc
            .cover_i
            .map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id)),
        description: None,
        publisher: doc.publisher.and_then(|p| p.into_iter().next()),
        published_date: doc.first_publish_year.map(|y| y.to_string()),
        page_count: doc.number_of_pages_median,
        isbn: prefer_isbn13(doc.isbn),
    };

    assert_eq!(meta.title.as_deref(), Some("Great Expectations"));
    assert_eq!(meta.author.as_deref(), Some("Charles Dickens"));
    assert_eq!(
        meta.cover_url.as_deref(),
        Some("https://covers.openlibrary.org/b/id/12345-L.jpg")
    );
    assert_eq!(meta.publisher.as_deref(), Some("Penguin"));
    assert_eq!(meta.published_date.as_deref(), Some("1861"));
    assert_eq!(meta.page_count, Some(544));
    assert_eq!(meta.isbn.as_deref(), Some("9780141439563"));
    assert!(meta.description.is_none());
}
