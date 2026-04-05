use crate::data::models::Highlight;
use crate::services::export::export_book_highlights_markdown;

fn make_highlight(
    text: &str,
    loc_start: Option<i64>,
    loc_end: Option<i64>,
    page: Option<i64>,
    clipped_at: Option<&str>,
) -> Highlight {
    Highlight {
        id: 1,
        book_id: 1,
        text: text.to_string(),
        location_start: loc_start,
        location_end: loc_end,
        page,
        clip_type: "highlight".to_string(),
        clipped_at: clipped_at.map(|s| s.to_string()),
        created_at: "2024-01-01".to_string(),
    }
}

#[test]
fn test_export_empty_highlights() {
    let md = export_book_highlights_markdown("Test Book", Some("Author"), &[]);
    assert!(md.contains("# Highlights from \"Test Book\" by Author"));
    // No quote blocks
    assert!(!md.contains("> "));
}

#[test]
fn test_export_single_highlight_with_location() {
    let highlights = vec![make_highlight(
        "Some text",
        Some(100),
        Some(200),
        None,
        Some("2024-01-15"),
    )];
    let md = export_book_highlights_markdown("My Book", Some("Jane Doe"), &highlights);

    assert!(md.contains("# Highlights from \"My Book\" by Jane Doe"));
    assert!(md.contains("> Some text"));
    assert!(md.contains("Location 100-200"));
    assert!(md.contains("Clipped on 2024-01-15"));
}

#[test]
fn test_export_highlight_with_page_only() {
    let highlights = vec![make_highlight("Page text", None, None, Some(42), None)];
    let md = export_book_highlights_markdown("Book", None, &highlights);

    assert!(md.contains("> Page text"));
    assert!(md.contains("Page 42"));
    assert!(!md.contains("Location"));
}

#[test]
fn test_export_skips_empty_text() {
    let highlights = vec![
        make_highlight("", None, None, None, None),
        make_highlight("Real text", None, None, None, None),
    ];
    let md = export_book_highlights_markdown("Book", None, &highlights);

    assert!(!md.contains("> \n"));
    assert!(md.contains("> Real text"));
}

#[test]
fn test_export_no_author() {
    let md = export_book_highlights_markdown("Solo Book", None, &[]);
    assert!(md.contains("# Highlights from \"Solo Book\""));
    assert!(!md.contains(" by "));
}

#[test]
fn test_export_location_start_only() {
    let highlights = vec![make_highlight("Text", Some(50), None, None, None)];
    let md = export_book_highlights_markdown("Book", None, &highlights);
    assert!(md.contains("Location 50"));
    assert!(!md.contains("Location 50-"));
}
