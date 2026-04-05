use urlencoding::encode;

use crate::data::models::Book;

pub struct PurchaseLink {
    pub name: String,
    pub url: String,
}

pub fn generate_links(book: &Book) -> Vec<PurchaseLink> {
    let mut links = Vec::new();

    // Amazon: prefer ASIN, fall back to ISBN search
    if let Some(asin) = book.asin.as_deref().filter(|s| !s.is_empty()) {
        links.push(PurchaseLink {
            name: "Amazon".into(),
            url: format!("https://www.amazon.com/dp/{asin}"),
        });
    } else if let Some(isbn) = book.isbn.as_deref().filter(|s| !s.is_empty()) {
        links.push(PurchaseLink {
            name: "Amazon".into(),
            url: format!("https://www.amazon.com/s?k={isbn}"),
        });
    }

    // ISBN-dependent links
    if let Some(isbn) = book.isbn.as_deref().filter(|s| !s.is_empty()) {
        links.push(PurchaseLink {
            name: "Bookshop.org".into(),
            url: format!("https://bookshop.org/p/books/{isbn}"),
        });
        links.push(PurchaseLink {
            name: "Open Library".into(),
            url: format!("https://openlibrary.org/isbn/{isbn}"),
        });
        links.push(PurchaseLink {
            name: "WorldCat".into(),
            url: format!("https://search.worldcat.org/search?q=bn:{isbn}"),
        });
    }

    // Google Books: always available via title + author search
    let query = format!(
        "{}{}",
        book.title,
        book.author
            .as_deref()
            .map(|a| format!(" {a}"))
            .unwrap_or_default()
    );
    links.push(PurchaseLink {
        name: "Google Books".into(),
        url: format!("https://www.google.com/search?tbm=bks&q={}", encode(&query)),
    });

    links
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_book(
        title: &str,
        author: Option<&str>,
        isbn: Option<&str>,
        asin: Option<&str>,
    ) -> Book {
        Book {
            id: 1,
            title: title.to_string(),
            author: author.map(String::from),
            isbn: isbn.map(String::from),
            asin: asin.map(String::from),
            cover_url: None,
            description: None,
            publisher: None,
            published_date: None,
            page_count: None,
            created_at: String::new(),
            updated_at: String::new(),
            rating: None,
            status: None,
            started_at: None,
            finished_at: None,
        }
    }

    #[test]
    fn test_asin_preferred_over_isbn_for_amazon() {
        let book = make_book("Test", Some("Author"), Some("1234567890"), Some("B00TEST"));
        let links = generate_links(&book);
        let amazon = links.iter().find(|l| l.name == "Amazon").unwrap();
        assert!(amazon.url.contains("/dp/B00TEST"));
    }

    #[test]
    fn test_isbn_fallback_for_amazon() {
        let book = make_book("Test", Some("Author"), Some("1234567890"), None);
        let links = generate_links(&book);
        let amazon = links.iter().find(|l| l.name == "Amazon").unwrap();
        assert!(amazon.url.contains("s?k=1234567890"));
    }

    #[test]
    fn test_isbn_links_present_when_isbn_exists() {
        let book = make_book("Test", Some("Author"), Some("1234567890"), None);
        let links = generate_links(&book);
        let names: Vec<&str> = links.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"Bookshop.org"));
        assert!(names.contains(&"Open Library"));
        assert!(names.contains(&"WorldCat"));
    }

    #[test]
    fn test_no_isbn_only_google_and_maybe_amazon() {
        let book = make_book("Test", Some("Author"), None, None);
        let links = generate_links(&book);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].name, "Google Books");
    }

    #[test]
    fn test_google_books_always_present() {
        let book = make_book("My Book", Some("Jane Doe"), None, None);
        let links = generate_links(&book);
        let google = links.iter().find(|l| l.name == "Google Books").unwrap();
        assert!(google.url.contains("My%20Book%20Jane%20Doe"));
    }
}
