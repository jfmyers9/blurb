use super::truncate_text;

#[test]
fn test_truncate_text_shorter_than_max() {
    assert_eq!(truncate_text("hello", 10), "hello");
}

#[test]
fn test_truncate_text_longer_than_max() {
    assert_eq!(truncate_text("hello world", 5), "hello...");
}

#[test]
fn test_truncate_text_multibyte_boundary() {
    // Emoji is a multi-byte char; slicing at byte boundary would panic without char-safe truncation
    assert_eq!(truncate_text("Hi 🌍 world", 5), "Hi 🌍 ...");
}

#[test]
fn test_truncate_text_empty() {
    assert_eq!(truncate_text("", 10), "");
}

#[test]
fn test_truncate_text_exactly_at_max() {
    assert_eq!(truncate_text("12345", 5), "12345");
}

#[test]
fn test_truncate_text_whitespace_only() {
    assert_eq!(truncate_text("   ", 10), "");
}
