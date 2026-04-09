use super::*;

#[test]
fn card_renders_at_3x_resolution() {
    let data = ShareCardData::Book {
        title: "Test".to_string(),
        author: "Author".to_string(),
        rating: None,
        cover_image_source: None,
        book_url: None,
    };
    let png = generate_card(&data).expect("should generate PNG");
    let img = image::load_from_memory(&png).expect("should decode PNG");
    assert_eq!(img.width(), 1200);
    assert_eq!(img.height(), 1800);
}

#[test]
fn book_card_with_fallback_cover_produces_valid_png() {
    let data = ShareCardData::Book {
        title: "The Great Gatsby".to_string(),
        author: "F. Scott Fitzgerald".to_string(),
        rating: Some(4),
        cover_image_source: None,
        book_url: None,
    };
    let png = generate_card(&data).expect("should generate PNG");
    assert!(png.len() > 100);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn highlight_card_with_long_quote_produces_valid_png() {
    let data = ShareCardData::Highlight {
        quote: "So we beat on, boats against the current, borne back ceaselessly into the past. \
                This is a longer quote to test word wrapping across multiple lines in the card."
            .to_string(),
        book_title: "The Great Gatsby".to_string(),
        author: "F. Scott Fitzgerald".to_string(),
        rating: Some(5),
    };
    let png = generate_card(&data).expect("should generate PNG");
    assert!(png.len() > 100);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn word_wrap_splits_on_boundaries() {
    let lines = word_wrap("hello world this is a test of wrapping", 15);
    assert_eq!(lines, vec!["hello world", "this is a test", "of wrapping"]);
}

#[test]
fn word_wrap_handles_empty_input() {
    let lines = word_wrap("", 35);
    assert!(lines.is_empty());
}

#[test]
fn word_wrap_single_long_word() {
    let lines = word_wrap("supercalifragilisticexpialidocious", 10);
    assert_eq!(lines, vec!["supercalifragilisticexpialidocious"]);
}

#[test]
fn highlight_card_without_rating() {
    let data = ShareCardData::Highlight {
        quote: "Short quote.".to_string(),
        book_title: "Test".to_string(),
        author: "Author".to_string(),
        rating: None,
    };
    let png = generate_card(&data).expect("should generate PNG");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn book_card_escapes_xml_in_title() {
    let data = ShareCardData::Book {
        title: "Tom & Jerry <Adventures>".to_string(),
        author: "Author".to_string(),
        rating: None,
        cover_image_source: None,
        book_url: None,
    };
    let png = generate_card(&data).expect("should generate PNG with special chars");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn book_card_long_title_wraps_without_overflow() {
    let data = ShareCardData::Book {
        title: "The Incredibly Long and Winding Title of a Book That Should Be Truncated"
            .to_string(),
        author: "Author Name".to_string(),
        rating: Some(3),
        cover_image_source: None,
        book_url: None,
    };
    let png = generate_card(&data).expect("should generate PNG for long title");
    assert!(png.len() > 100);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn highlight_card_long_quote_clamped_to_max_lines() {
    let sentence = "This is a fairly long sentence that will repeat many times to create a very long quote for testing purposes. ";
    let long_quote = sentence.repeat(8);
    let data = ShareCardData::Highlight {
        quote: long_quote,
        book_title: "Test Book".to_string(),
        author: "Author".to_string(),
        rating: Some(4),
    };
    let png = generate_card(&data).expect("should generate PNG for long quote");
    assert!(png.len() > 100);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}
