use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[cfg(test)]
#[path = "goodreads_tests.rs"]
mod tests;

#[derive(Debug, Clone, Deserialize)]
struct CsvRow {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Author")]
    author: String,
    #[serde(rename = "ISBN")]
    isbn: String,
    #[serde(rename = "ISBN13")]
    isbn13: String,
    #[serde(rename = "My Rating")]
    my_rating: String,
    #[serde(rename = "Number of Pages")]
    page_count: String,
    #[serde(rename = "Publisher")]
    publisher: String,
    #[serde(rename = "Year Published")]
    published_year: String,
    #[serde(rename = "Exclusive Shelf")]
    exclusive_shelf: String,
    #[serde(rename = "Date Read")]
    date_read: String,
    #[serde(rename = "Date Added")]
    date_added: String,
    #[serde(rename = "My Review")]
    my_review: String,
    #[serde(rename = "Bookshelves")]
    bookshelves: String,
}

#[derive(Debug, Clone)]
pub struct GoodreadsBook {
    pub title: String,
    pub author: String,
    pub isbn: Option<String>,
    pub isbn13: Option<String>,
    pub rating: Option<i32>,
    pub page_count: Option<i32>,
    pub publisher: Option<String>,
    pub published_year: Option<String>,
    pub status: String,
    pub date_read: Option<String>,
    pub date_added: Option<String>,
    pub review_text: Option<String>,
    pub bookshelves: Vec<String>,
}

pub fn parse_goodreads_csv(path: &Path) -> Result<Vec<GoodreadsBook>> {
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_path(path)?;

    let mut books = Vec::new();
    for result in reader.deserialize() {
        let row: CsvRow = result?;
        books.push(GoodreadsBook {
            title: row.title,
            author: row.author,
            isbn: unwrap_isbn(&row.isbn),
            isbn13: unwrap_isbn(&row.isbn13),
            rating: parse_rating(&row.my_rating),
            page_count: parse_page_count(&row.page_count),
            publisher: non_empty(row.publisher),
            published_year: non_empty(row.published_year),
            status: map_shelf(&row.exclusive_shelf),
            date_read: convert_date(&row.date_read),
            date_added: convert_date(&row.date_added),
            review_text: non_empty(row.my_review),
            bookshelves: parse_bookshelves(&row.bookshelves),
        });
    }
    Ok(books)
}

/// Strips the `="..."` wrapper Goodreads uses for ISBN fields.
fn unwrap_isbn(raw: &str) -> Option<String> {
    let inner = raw
        .strip_prefix("=\"")
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw);
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}

fn convert_date(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return None;
    }
    Some(raw.replace('/', "-"))
}

fn map_shelf(shelf: &str) -> String {
    match shelf {
        "read" => "finished",
        "currently-reading" => "reading",
        _ => "want_to_read",
    }
    .to_string()
}

fn parse_rating(raw: &str) -> Option<i32> {
    raw.parse::<i32>().ok().filter(|r| (1..=5).contains(r))
}

fn parse_page_count(raw: &str) -> Option<i32> {
    raw.parse::<i32>().ok().filter(|&n| n > 0)
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn parse_bookshelves(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split(", ").map(|s| s.to_string()).collect()
}
