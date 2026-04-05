use super::*;

#[test]
fn test_parse_highlight() {
    let input = r#"The Great Gatsby (F. Scott Fitzgerald)
- Your Highlight on Location 234-238 | Added on Friday, March 15, 2024 2:30:15 PM

So we beat on, boats against the current, borne back ceaselessly into the past.
=========="#;
    let clips = parse_clippings(input);
    assert_eq!(clips.len(), 1);
    let c = &clips[0];
    assert_eq!(c.title, "The Great Gatsby");
    assert_eq!(c.author.as_deref(), Some("F. Scott Fitzgerald"));
    assert_eq!(c.clip_type, "highlight");
    assert_eq!(c.location_start, Some(234));
    assert_eq!(c.location_end, Some(238));
    assert!(c.text.contains("So we beat on"));
}

#[test]
fn test_parse_note() {
    let input = r#"1984 (George Orwell)
- Your Note on page 42 | Added on Saturday, April 20, 2024 10:15:00 AM

This reminds me of modern surveillance
=========="#;
    let clips = parse_clippings(input);
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].clip_type, "note");
    assert_eq!(clips[0].page, Some(42));
}

#[test]
fn test_parse_bookmark() {
    let input = r#"Dune (Frank Herbert)
- Your Bookmark on Location 1500 | Added on Sunday, May 5, 2024 8:00:00 AM


=========="#;
    let clips = parse_clippings(input);
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].clip_type, "bookmark");
    assert_eq!(clips[0].location_start, Some(1500));
    assert!(clips[0].text.is_empty());
}

#[test]
fn test_parse_timestamp_midnight() {
    let block = "Test Book (Author)\n- Your Highlight on page 1 | Location 1-2 | Added on Monday, January 1, 2024 12:00:00 AM\n\nSome text";
    let clippings = parse_clippings(&format!("{}\n==========", block));
    assert_eq!(clippings.len(), 1);
    assert!(
        clippings[0]
            .clipped_at
            .as_ref()
            .unwrap()
            .contains("T00:00:00"),
        "12:00:00 AM should map to hour 0, got: {}",
        clippings[0].clipped_at.as_ref().unwrap()
    );
}

#[test]
fn test_parse_timestamp_noon() {
    let block = "Test Book (Author)\n- Your Highlight on page 1 | Location 1-2 | Added on Monday, January 1, 2024 12:30:00 PM\n\nSome text";
    let clippings = parse_clippings(&format!("{}\n==========", block));
    assert_eq!(clippings.len(), 1);
    assert!(
        clippings[0]
            .clipped_at
            .as_ref()
            .unwrap()
            .contains("T12:30:00"),
        "12:30:00 PM should stay hour 12, got: {}",
        clippings[0].clipped_at.as_ref().unwrap()
    );
}
