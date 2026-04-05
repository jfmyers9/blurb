use serde::Deserialize;
use std::collections::HashSet;
use std::io::Read;

/// Represents a single row from a Goodreads CSV export.
/// Only fields we actually use are kept; unknown columns are ignored via `deny_unknown_fields = false`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GoodreadsRecord {
    #[serde(rename = "Title", default)]
    pub title: Option<String>,
    #[serde(rename = "Author", default)]
    pub author: Option<String>,
    #[serde(rename = "ISBN", default)]
    pub isbn: Option<String>,
    #[serde(rename = "ISBN13", default)]
    pub isbn13: Option<String>,
    #[serde(rename = "My Rating", default)]
    pub my_rating: Option<String>,
    #[serde(rename = "Publisher", default)]
    pub publisher: Option<String>,
    #[serde(rename = "Number of Pages", default)]
    pub number_of_pages: Option<String>,
    #[serde(rename = "Year Published", default)]
    pub year_published: Option<String>,
    #[serde(rename = "Original Publication Year", default)]
    pub original_publication_year: Option<String>,
    #[serde(rename = "Date Read", default)]
    pub date_read: Option<String>,
    #[serde(rename = "Date Added", default)]
    pub date_added: Option<String>,
    #[serde(rename = "Bookshelves", default)]
    pub bookshelves: Option<String>,
    #[serde(rename = "Exclusive Shelf", default)]
    pub exclusive_shelf: Option<String>,
    #[serde(rename = "My Review", default)]
    pub my_review: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedGoodreadsBook {
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub page_count: Option<i32>,
    pub published_date: Option<String>,
    pub rating: Option<i32>,
    pub status: String,
    pub date_read: Option<String>,
    pub date_added: Option<String>,
    pub review: Option<String>,
    pub shelves: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GoodreadsParseResult {
    pub books: Vec<ParsedGoodreadsBook>,
    pub skipped_rows: usize,
}

fn clean_isbn(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == 'X')
        .collect();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn map_exclusive_shelf(shelf: &str) -> String {
    match shelf.trim() {
        "read" => "finished".to_string(),
        "currently-reading" => "reading".to_string(),
        "to-read" => "want_to_read".to_string(),
        _ => "want_to_read".to_string(),
    }
}

fn parse_goodreads_date(date_str: &str) -> Option<String> {
    let trimmed = date_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Goodreads dates are typically YYYY/MM/DD
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() == 3 {
        let year = parts[0];
        let month = parts[1];
        let day = parts[2];
        if year.len() == 4
            && year.chars().all(|c| c.is_ascii_digit())
            && month.chars().all(|c| c.is_ascii_digit())
            && day.chars().all(|c| c.is_ascii_digit())
        {
            return Some(format!("{}-{:0>2}-{:0>2}", year, month, day));
        }
    }
    None
}

fn parse_shelves(bookshelves: &str) -> Vec<String> {
    bookshelves
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn parse_goodreads_csv<R: Read>(reader: R) -> Result<GoodreadsParseResult, String> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);

    let mut books = Vec::new();
    let mut skipped_rows = 0usize;

    for result in csv_reader.deserialize::<GoodreadsRecord>() {
        let record = match result {
            Ok(r) => r,
            Err(_) => {
                skipped_rows += 1;
                continue;
            }
        };

        let title = match record.title.and_then(|t| non_empty(&t)) {
            Some(t) => t,
            None => {
                skipped_rows += 1;
                continue;
            }
        };

        let author = record.author.and_then(|a| non_empty(&a));

        // Prefer ISBN13 over ISBN
        let isbn = record
            .isbn13
            .and_then(|i| clean_isbn(&i))
            .or_else(|| record.isbn.and_then(|i| clean_isbn(&i)));

        let publisher = record.publisher.and_then(|p| non_empty(&p));

        let page_count = record
            .number_of_pages
            .and_then(|n| n.trim().parse::<i32>().ok());

        let published_date = record
            .original_publication_year
            .and_then(|y| non_empty(&y))
            .or_else(|| record.year_published.and_then(|y| non_empty(&y)))
            .map(|y| format!("{}-01-01", y.trim()));

        let rating = record
            .my_rating
            .and_then(|r| r.trim().parse::<i32>().ok())
            .filter(|r| (1..=5).contains(r));

        let status = record
            .exclusive_shelf
            .as_deref()
            .map(map_exclusive_shelf)
            .unwrap_or_else(|| "want_to_read".to_string());

        let date_read = record.date_read.as_deref().and_then(parse_goodreads_date);
        let date_added = record.date_added.as_deref().and_then(parse_goodreads_date);

        let review = record.my_review.and_then(|r| non_empty(&r));

        let shelves = record
            .bookshelves
            .as_deref()
            .map(parse_shelves)
            .unwrap_or_default();

        books.push(ParsedGoodreadsBook {
            title,
            author,
            isbn,
            publisher,
            page_count,
            published_date,
            rating,
            status,
            date_read,
            date_added,
            review,
            shelves,
        });
    }

    Ok(GoodreadsParseResult {
        books,
        skipped_rows,
    })
}

pub fn collect_unique_shelves(books: &[ParsedGoodreadsBook]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut shelves = Vec::new();
    for book in books {
        for shelf in &book.shelves {
            if seen.insert(shelf.clone()) {
                shelves.push(shelf.clone());
            }
        }
    }
    shelves.sort();
    shelves
}

#[cfg(test)]
#[path = "goodreads_tests.rs"]
mod tests;
