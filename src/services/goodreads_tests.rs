use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

const CSV_HEADER: &str = "Book Id,Title,Author,Author l-f,Additional Authors,ISBN,ISBN13,My Rating,Average Rating,Publisher,Binding,Number of Pages,Year Published,Original Publication Year,Date Read,Date Added,Bookshelves,Bookshelves with positions,Exclusive Shelf,My Review,Spoiler,Private Notes,Read Count,Owned Copies";

// 24 columns: 0-BookId 1-Title 2-Author 3-AuthorLF 4-AddlAuthors 5-ISBN 6-ISBN13
// 7-MyRating 8-AvgRating 9-Publisher 10-Binding 11-NumPages 12-YearPub 13-OrigPubYear
// 14-DateRead 15-DateAdded 16-Bookshelves 17-BookshelvesPos 18-ExclusiveShelf
// 19-MyReview 20-Spoiler 21-PrivateNotes 22-ReadCount 23-OwnedCopies

fn parse_single_row(row: &str) -> GoodreadsBook {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{CSV_HEADER}").unwrap();
    writeln!(file, "{row}").unwrap();
    let books = parse_goodreads_csv(file.path()).unwrap();
    assert_eq!(books.len(), 1);
    books.into_iter().next().unwrap()
}

#[test]
fn isbn_with_value_is_extracted() {
    let book = parse_single_row(
        r#"1,Title,Author,"Last, First",,"=""0593983769""","=""9780593983768""",0,4.0,,,0,,,,,,,,,,,,0"#,
    );
    assert_eq!(book.isbn.as_deref(), Some("0593983769"));
    assert_eq!(book.isbn13.as_deref(), Some("9780593983768"));
}

#[test]
fn empty_isbn_becomes_none() {
    let book = parse_single_row(
        r#"1,Title,Author,"Last, First",,"=""""","=""""",0,4.0,,,0,,,,,,,to-read,,,,0,0"#,
    );
    assert_eq!(book.isbn, None);
    assert_eq!(book.isbn13, None);
}

#[test]
fn date_converted_to_iso() {
    let book = parse_single_row(
        r#"1,Title,Author,"Last, First",,"=""""","=""""",0,4.0,,,0,,,2026/03/28,2026/01/15,,,read,,,,1,0"#,
    );
    assert_eq!(book.date_read.as_deref(), Some("2026-03-28"));
    assert_eq!(book.date_added.as_deref(), Some("2026-01-15"));
}

#[test]
fn empty_dates_become_none() {
    let book = parse_single_row(
        r#"1,Title,Author,"Last, First",,"=""""","=""""",0,4.0,,,0,,,,,,,read,,,,0,0"#,
    );
    assert_eq!(book.date_read, None);
    assert_eq!(book.date_added, None);
}

#[test]
fn shelf_mapping() {
    let read = parse_single_row(r#"1,T,A,"L, F",,"=""""","=""""",0,0,,,0,,,,,,,read,,,,0,0"#);
    let reading =
        parse_single_row(r#"1,T,A,"L, F",,"=""""","=""""",0,0,,,0,,,,,,,currently-reading,,,,0,0"#);
    let to_read = parse_single_row(r#"1,T,A,"L, F",,"=""""","=""""",0,0,,,0,,,,,,,to-read,,,,0,0"#);
    assert_eq!(read.status, "finished");
    assert_eq!(reading.status, "reading");
    assert_eq!(to_read.status, "want_to_read");
}

#[test]
fn zero_rating_becomes_none() {
    let book = parse_single_row(r#"1,T,A,"L, F",,"=""""","=""""",0,0,,,0,,,,,,,read,,,,0,0"#);
    assert_eq!(book.rating, None);
}

#[test]
fn nonzero_rating_preserved() {
    let book = parse_single_row(r#"1,T,A,"L, F",,"=""""","=""""",4,0,,,0,,,,,,,read,,,,0,0"#);
    assert_eq!(book.rating, Some(4));
}

#[test]
fn bookshelves_split() {
    let book = parse_single_row(
        r#"1,T,A,"L, F",,"=""""","=""""",0,0,,,0,,,,,"fiction, favorites",,read,,,,0,0"#,
    );
    assert_eq!(book.bookshelves, vec!["fiction", "favorites"]);
}

#[test]
fn empty_bookshelves() {
    let book = parse_single_row(r#"1,T,A,"L, F",,"=""""","=""""",0,0,,,0,,,,,,,read,,,,0,0"#);
    assert!(book.bookshelves.is_empty());
}

#[test]
fn zero_page_count_becomes_none() {
    let book = parse_single_row(r#"1,T,A,"L, F",,"=""""","=""""",0,0,,,0,,,,,,,read,,,,0,0"#);
    assert_eq!(book.page_count, None);
}

#[test]
fn full_sample_row() {
    let book = parse_single_row(
        r#"38465292,The Story of a New Name (Neapolitan Novels #2),Elena Ferrante,"Ferrante, Elena",Ann Goldstein,"=""""","=""""",4,4.47,Europa Editions,Kindle Edition,571,2013,2012,2026/03/28,2026/03/22,,,read,,,,1,0"#,
    );
    assert_eq!(book.title, "The Story of a New Name (Neapolitan Novels #2)");
    assert_eq!(book.author, "Elena Ferrante");
    assert_eq!(book.isbn, None);
    assert_eq!(book.rating, Some(4));
    assert_eq!(book.page_count, Some(571));
    assert_eq!(book.publisher.as_deref(), Some("Europa Editions"));
    assert_eq!(book.published_year.as_deref(), Some("2013"));
    assert_eq!(book.status, "finished");
    assert_eq!(book.date_read.as_deref(), Some("2026-03-28"));
    assert_eq!(book.date_added.as_deref(), Some("2026-03-22"));
}
