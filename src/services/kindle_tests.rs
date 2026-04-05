use super::*;

#[test]
fn strip_asin_underscore() {
    assert_eq!(
        strip_asin_suffix("Title_B0ABCDEFGH"),
        ("Title", Some("B0ABCDEFGH"))
    );
}

#[test]
fn strip_asin_parens() {
    assert_eq!(
        strip_asin_suffix("Title (B0ABCDEFGH)"),
        ("Title", Some("B0ABCDEFGH"))
    );
}

#[test]
fn strip_ebok() {
    assert_eq!(strip_asin_suffix("Title_EBOK"), ("Title", None));
}

#[test]
fn strip_pdoc() {
    assert_eq!(strip_asin_suffix("Title_PDOC"), ("Title", None));
}

#[test]
fn strip_ebsp() {
    assert_eq!(strip_asin_suffix("Title_EBSP"), ("Title", None));
}

#[test]
fn strip_no_match() {
    assert_eq!(strip_asin_suffix("Title"), ("Title", None));
}

#[test]
fn parse_title_and_author() {
    let (title, author, asin) = parse_kindle_filename("The Great Gatsby - F Scott Fitzgerald");
    assert_eq!(title, "The Great Gatsby");
    assert_eq!(author, Some("F Scott Fitzgerald".to_string()));
    assert_eq!(asin, None);
}

#[test]
fn parse_underscores_no_author() {
    let (title, author, asin) = parse_kindle_filename("Some_Book_Title");
    assert_eq!(title, "Some Book Title");
    assert_eq!(author, None);
    assert_eq!(asin, None);
}

#[test]
fn parse_empty_author() {
    let (title, author, asin) = parse_kindle_filename("Title - ");
    assert_eq!(title, "Title");
    assert_eq!(author, None);
    assert_eq!(asin, None);
}

#[test]
fn parse_strips_asin_from_both() {
    let (title, author, asin) = parse_kindle_filename("My Book - Author_B0ABCDEFGH");
    assert_eq!(title, "My Book");
    assert_eq!(author, Some("Author".to_string()));
    assert_eq!(asin, Some("B0ABCDEFGH".to_string()));
}

#[test]
fn strip_asin_multibyte_no_panic() {
    let (name, asin) = strip_asin_suffix("日本語_B0ABCDEFGH");
    assert_eq!(name, "日本語");
    assert_eq!(asin, Some("B0ABCDEFGH"));
}
