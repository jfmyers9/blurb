use super::*;

#[test]
fn test_stars_full() {
    assert_eq!(stars(5), "\u{2605}\u{2605}\u{2605}\u{2605}\u{2605}");
}

#[test]
fn test_stars_partial() {
    assert_eq!(stars(3), "\u{2605}\u{2605}\u{2605}\u{2606}\u{2606}");
}

#[test]
fn test_stars_zero() {
    assert_eq!(stars(0), "\u{2606}\u{2606}\u{2606}\u{2606}\u{2606}");
}

#[test]
fn test_stars_clamps_above_five() {
    assert_eq!(stars(10), "\u{2605}\u{2605}\u{2605}\u{2605}\u{2605}");
}

#[test]
fn test_stars_clamps_below_zero() {
    assert_eq!(stars(-3), "\u{2606}\u{2606}\u{2606}\u{2606}\u{2606}");
}

#[test]
fn test_format_book_share_full() {
    let result = format_book_share(
        "Project Hail Mary",
        Some("Andy Weir"),
        Some(5),
        Some("reading"),
    );
    assert!(result.contains("Currently reading"));
    assert!(result.contains("Project Hail Mary"));
    assert!(result.contains("by Andy Weir"));
    assert!(result.contains("\u{2605}\u{2605}\u{2605}\u{2605}\u{2605}"));
}

#[test]
fn test_format_book_share_no_author() {
    let result = format_book_share("Untitled", None, None, Some("want_to_read"));
    assert!(result.contains("Want to read"));
    assert!(result.contains("Untitled"));
    assert!(!result.contains("by "));
}

#[test]
fn test_format_book_share_finished() {
    let result = format_book_share("Dune", Some("Frank Herbert"), Some(4), Some("finished"));
    assert!(result.contains("Finished"));
}

#[test]
fn test_format_book_share_unknown_status() {
    let result = format_book_share("Book", None, None, Some("custom"));
    assert!(result.contains("Reading"));
}

#[test]
fn test_format_book_share_no_status() {
    let result = format_book_share("Book", None, None, None);
    assert!(result.contains("Reading"));
}

#[test]
fn test_format_diary_share_full() {
    let result = format_diary_share(
        "Project Hail Mary",
        Some("Andy Weir"),
        Some(5),
        "April 4, 2026",
        Some("The prose was beautiful"),
    );
    assert!(result.contains("Finished: Project Hail Mary by Andy Weir"));
    assert!(result.contains("\u{2605}\u{2605}\u{2605}\u{2605}\u{2605} | April 4, 2026"));
    assert!(result.contains("\"The prose was beautiful\""));
}

#[test]
fn test_format_diary_share_no_body() {
    let result = format_diary_share("Dune", None, Some(4), "March 1, 2026", None);
    assert!(!result.contains('"'));
}

#[test]
fn test_format_diary_share_empty_body() {
    let result = format_diary_share("Dune", None, Some(4), "March 1, 2026", Some(""));
    assert!(!result.contains("\"\""));
}

#[test]
fn test_format_diary_share_no_rating() {
    let result = format_diary_share("Dune", None, None, "March 1, 2026", None);
    assert!(result.contains("\nMarch 1, 2026"));
}

#[test]
fn test_format_shelf_share() {
    let books = vec![
        (
            "Project Hail Mary".to_string(),
            Some("Andy Weir".to_string()),
        ),
        ("Dune".to_string(), Some("Frank Herbert".to_string())),
        ("Neuromancer".to_string(), None),
    ];
    let result = format_shelf_share("Science Fiction", &books);
    assert!(result.contains("My Shelf: Science Fiction"));
    assert!(result.contains("1. Project Hail Mary \u{2014} Andy Weir"));
    assert!(result.contains("2. Dune \u{2014} Frank Herbert"));
    assert!(result.contains("3. Neuromancer"));
    assert!(!result.contains("3. Neuromancer \u{2014}"));
}

#[test]
fn test_format_shelf_share_empty() {
    let result = format_shelf_share("Empty Shelf", &[]);
    assert_eq!(result, "\u{1f4da} My Shelf: Empty Shelf");
}

#[test]
fn test_format_stats_share_full() {
    let result = format_stats_share(42, 30, 15234, Some(4.3));
    assert!(result.contains("42 books"));
    assert!(result.contains("30 finished"));
    assert!(result.contains("15234 pages"));
    assert!(result.contains("Avg \u{2605}\u{2605}\u{2605}\u{2605}\u{2606}"));
}

#[test]
fn test_format_stats_share_no_pages() {
    let result = format_stats_share(5, 2, 0, None);
    assert!(!result.contains("pages"));
    assert!(!result.contains("Avg"));
}

#[test]
fn test_format_stats_share_no_rating() {
    let result = format_stats_share(10, 5, 1000, None);
    assert!(result.contains("1000 pages"));
    assert!(!result.contains("Avg"));
}
