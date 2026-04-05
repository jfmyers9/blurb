use super::*;

fn sample_csv() -> &'static str {
    "Book Id,Title,Author,Author l-f,Additional Authors,ISBN,ISBN13,My Rating,Average Rating,Publisher,Binding,Number of Pages,Year Published,Original Publication Year,Date Read,Date Added,Bookshelves,Bookshelves with positions,Exclusive Shelf,My Review,Spoiler,Private Notes,Read Count,Owned Copies\n\
     12345,The Great Gatsby,F. Scott Fitzgerald,\"Fitzgerald, F. Scott\",,=\"0743273567\",=\"9780743273565\",5,3.93,Scribner,Paperback,180,2004,1925,2023/06/15,2023/01/10,\"classics, fiction\",,read,Amazing book,,,,1,0\n\
     67890,Dune,Frank Herbert,\"Herbert, Frank\",,=\"0441172717\",=\"9780441172719\",4,4.25,Ace Books,Mass Market Paperback,688,2005,1965,2023/08/20,2023/07/01,\"sci-fi, favorites\",,read,,,,,1,1\n\
     11111,Project Hail Mary,Andy Weir,\"Weir, Andy\",,,,0,,Ballantine Books,Hardcover,496,2021,2021,,,,,currently-reading,,,,,0,0\n\
     22222,The Name of the Wind,Patrick Rothfuss,\"Rothfuss, Patrick\",,,,0,,DAW Books,Paperback,662,2007,2007,,,,,to-read,,,,,0,0"
}

#[test]
fn test_parse_goodreads_csv() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    assert_eq!(result.skipped_rows, 0);
    assert_eq!(result.books.len(), 4);
}

#[test]
fn test_parse_title_and_author() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    let book = &result.books[0];
    assert_eq!(book.title, "The Great Gatsby");
    assert_eq!(book.author.as_deref(), Some("F. Scott Fitzgerald"));
}

#[test]
fn test_parse_isbn() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    // Should prefer ISBN13
    assert_eq!(result.books[0].isbn.as_deref(), Some("9780743273565"));
    assert_eq!(result.books[1].isbn.as_deref(), Some("9780441172719"));
    // No ISBN
    assert!(result.books[2].isbn.is_none());
}

#[test]
fn test_parse_rating() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    assert_eq!(result.books[0].rating, Some(5));
    assert_eq!(result.books[1].rating, Some(4));
    // Rating 0 should be filtered out
    assert!(result.books[2].rating.is_none());
}

#[test]
fn test_parse_status() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    assert_eq!(result.books[0].status, "finished");
    assert_eq!(result.books[1].status, "finished");
    assert_eq!(result.books[2].status, "reading");
    assert_eq!(result.books[3].status, "want_to_read");
}

#[test]
fn test_parse_dates() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    assert_eq!(result.books[0].date_read.as_deref(), Some("2023-06-15"));
    assert_eq!(result.books[0].date_added.as_deref(), Some("2023-01-10"));
    assert!(result.books[2].date_read.is_none());
}

#[test]
fn test_parse_shelves() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    assert_eq!(result.books[0].shelves, vec!["classics", "fiction"]);
    assert_eq!(result.books[1].shelves, vec!["sci-fi", "favorites"]);
    assert!(result.books[3].shelves.is_empty());
}

#[test]
fn test_parse_review() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    assert_eq!(result.books[0].review.as_deref(), Some("Amazing book"));
    assert!(result.books[2].review.is_none());
}

#[test]
fn test_collect_unique_shelves() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    let shelves = collect_unique_shelves(&result.books);
    assert_eq!(shelves, vec!["classics", "favorites", "fiction", "sci-fi"]);
}

#[test]
fn test_parse_published_date() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    // Should use Original Publication Year
    assert_eq!(
        result.books[0].published_date.as_deref(),
        Some("1925-01-01")
    );
}

#[test]
fn test_parse_page_count() {
    let result = parse_goodreads_csv(sample_csv().as_bytes()).unwrap();
    assert_eq!(result.books[0].page_count, Some(180));
    assert_eq!(result.books[1].page_count, Some(688));
}

#[test]
fn test_clean_isbn_strips_equals_quotes() {
    assert_eq!(
        clean_isbn("=\"0743273567\""),
        Some("0743273567".to_string())
    );
    assert_eq!(
        clean_isbn("9780743273565"),
        Some("9780743273565".to_string())
    );
    assert_eq!(clean_isbn("=\"\""), None);
    assert_eq!(clean_isbn(""), None);
}

#[test]
fn test_parse_goodreads_date() {
    assert_eq!(
        parse_goodreads_date("2023/06/15"),
        Some("2023-06-15".to_string())
    );
    assert_eq!(
        parse_goodreads_date("2023/1/5"),
        Some("2023-01-05".to_string())
    );
    assert_eq!(parse_goodreads_date(""), None);
    assert_eq!(parse_goodreads_date("not-a-date"), None);
}

#[test]
fn test_empty_csv() {
    let csv = "Book Id,Title,Author,Author l-f,Additional Authors,ISBN,ISBN13,My Rating,Average Rating,Publisher,Binding,Number of Pages,Year Published,Original Publication Year,Date Read,Date Added,Bookshelves,Bookshelves with positions,Exclusive Shelf,My Review,Spoiler,Private Notes,Read Count,Owned Copies\n";
    let result = parse_goodreads_csv(csv.as_bytes()).unwrap();
    assert_eq!(result.books.len(), 0);
    assert_eq!(result.skipped_rows, 0);
}

#[test]
fn test_row_without_title_skipped() {
    let csv = "Book Id,Title,Author,Author l-f,Additional Authors,ISBN,ISBN13,My Rating,Average Rating,Publisher,Binding,Number of Pages,Year Published,Original Publication Year,Date Read,Date Added,Bookshelves,Bookshelves with positions,Exclusive Shelf,My Review,Spoiler,Private Notes,Read Count,Owned Copies\n\
               12345,,SomeAuthor,,,,,0,,,,,,,,,,,,,,,,0,0\n";
    let result = parse_goodreads_csv(csv.as_bytes()).unwrap();
    assert_eq!(result.books.len(), 0);
    assert_eq!(result.skipped_rows, 1);
}
