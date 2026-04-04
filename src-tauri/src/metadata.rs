use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub page_count: Option<i32>,
    pub isbn: Option<String>,
}

fn sanitize_isbn(isbn: &str) -> String {
    isbn.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

pub async fn lookup(isbn: &str) -> Result<BookMetadata, String> {
    let isbn = sanitize_isbn(isbn);
    if isbn.is_empty() {
        return Err("ISBN is empty".into());
    }

    if let Ok(meta) = open_library(&isbn).await {
        if meta.title.is_some() {
            return Ok(meta);
        }
    }

    google_books(&isbn).await
}

#[derive(Deserialize)]
struct OLBookData {
    title: Option<String>,
    publishers: Option<Vec<OLPublisher>>,
    publish_date: Option<String>,
    number_of_pages: Option<i32>,
    authors: Option<Vec<OLAuthor>>,
    #[serde(default)]
    excerpts: Option<Vec<OLExcerpt>>,
}

#[derive(Deserialize)]
struct OLPublisher {
    name: Option<String>,
}

#[derive(Deserialize)]
struct OLAuthor {
    name: Option<String>,
}

#[derive(Deserialize)]
struct OLExcerpt {
    text: Option<String>,
}

async fn open_library(isbn: &str) -> Result<BookMetadata, String> {
    let url = format!(
        "https://openlibrary.org/api/books?bibkeys=ISBN:{}&format=json&jscmd=data",
        isbn
    );
    let client = reqwest::Client::new();
    let resp: std::collections::HashMap<String, OLBookData> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let key = format!("ISBN:{}", isbn);
    let data = resp.get(&key).ok_or("ISBN not found on Open Library")?;

    Ok(BookMetadata {
        title: data.title.clone(),
        author: data
            .authors
            .as_ref()
            .and_then(|a| a.first())
            .and_then(|a| a.name.clone()),
        cover_url: Some(format!(
            "https://covers.openlibrary.org/b/isbn/{}-L.jpg",
            isbn
        )),
        description: data
            .excerpts
            .as_ref()
            .and_then(|e| e.first())
            .and_then(|e| e.text.clone()),
        publisher: data
            .publishers
            .as_ref()
            .and_then(|p| p.first())
            .and_then(|p| p.name.clone()),
        published_date: data.publish_date.clone(),
        page_count: data.number_of_pages,
        isbn: Some(isbn.to_string()),
    })
}

#[derive(Deserialize)]
struct GoogleBooksResponse {
    items: Option<Vec<GoogleBooksItem>>,
}

#[derive(Deserialize)]
struct GoogleBooksItem {
    #[serde(rename = "volumeInfo")]
    volume_info: GoogleVolumeInfo,
}

#[derive(Deserialize)]
struct GoogleVolumeInfo {
    title: Option<String>,
    authors: Option<Vec<String>>,
    publisher: Option<String>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
    #[serde(rename = "pageCount")]
    page_count: Option<i32>,
    description: Option<String>,
    #[serde(rename = "imageLinks")]
    image_links: Option<GoogleImageLinks>,
    #[serde(rename = "industryIdentifiers")]
    industry_identifiers: Option<Vec<GoogleIndustryIdentifier>>,
}

#[derive(Deserialize)]
struct GoogleIndustryIdentifier {
    #[serde(rename = "type")]
    id_type: String,
    identifier: String,
}

#[derive(Deserialize)]
struct GoogleImageLinks {
    thumbnail: Option<String>,
}

#[derive(Deserialize)]
struct OLSearchResponse {
    docs: Option<Vec<OLSearchDoc>>,
}

#[derive(Deserialize)]
struct OLSearchDoc {
    title: Option<String>,
    author_name: Option<Vec<String>>,
    cover_i: Option<i64>,
    #[allow(dead_code)]
    isbn: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct OLTitleSearchResponse {
    docs: Option<Vec<OLTitleSearchDoc>>,
}

#[derive(Deserialize)]
struct OLTitleSearchDoc {
    title: Option<String>,
    author_name: Option<Vec<String>>,
    cover_i: Option<i64>,
    isbn: Option<Vec<String>>,
    publisher: Option<Vec<String>>,
    first_publish_year: Option<i32>,
    number_of_pages_median: Option<i32>,
}

pub async fn search_covers(query: &str) -> Result<Vec<BookMetadata>, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://openlibrary.org/search.json?q={}&fields=title,author_name,cover_i,isbn&limit=5",
        urlencoding::encode(query)
    );
    let resp: OLSearchResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let results = resp
        .docs
        .unwrap_or_default()
        .into_iter()
        .map(|doc| BookMetadata {
            title: doc.title,
            author: doc.author_name.as_ref().and_then(|a| a.first().cloned()),
            cover_url: doc
                .cover_i
                .map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id)),
            description: None,
            publisher: None,
            published_date: None,
            page_count: None,
            isbn: None,
        })
        .collect();

    Ok(results)
}

pub async fn search_by_title(title: &str, author: Option<&str>) -> Result<BookMetadata, String> {
    // Try Open Library search first
    if let Ok(meta) = search_by_title_open_library(title, author).await {
        if meta.title.is_some() {
            return Ok(meta);
        }
    }

    // Fall back to Google Books title search
    search_by_title_google(title, author).await
}

async fn search_by_title_open_library(
    title: &str,
    author: Option<&str>,
) -> Result<BookMetadata, String> {
    let query = match author {
        Some(a) => format!("{} {}", title, a),
        None => title.to_string(),
    };
    let url = format!(
        "https://openlibrary.org/search.json?q={}&fields=title,author_name,cover_i,isbn,publisher,first_publish_year,number_of_pages_median&limit=1",
        urlencoding::encode(&query)
    );
    let client = reqwest::Client::new();
    let resp: OLTitleSearchResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let doc = resp
        .docs
        .and_then(|d| d.into_iter().next())
        .ok_or("No results from Open Library")?;

    let isbn = doc.isbn.as_ref().and_then(|isbns| isbns.first().cloned());
    let cover_url = doc
        .cover_i
        .map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id));

    Ok(BookMetadata {
        title: doc.title,
        author: doc.author_name.as_ref().and_then(|a| a.first().cloned()),
        cover_url,
        description: None,
        publisher: doc.publisher.as_ref().and_then(|p| p.first().cloned()),
        published_date: doc.first_publish_year.map(|y| y.to_string()),
        page_count: doc.number_of_pages_median,
        isbn,
    })
}

async fn search_by_title_google(
    title: &str,
    author: Option<&str>,
) -> Result<BookMetadata, String> {
    let query = match author {
        Some(a) => format!("intitle:{}+inauthor:{}", urlencoding::encode(title), urlencoding::encode(a)),
        None => format!("intitle:{}", urlencoding::encode(title)),
    };
    let url = format!(
        "https://www.googleapis.com/books/v1/volumes?q={}",
        query
    );
    let client = reqwest::Client::new();
    let resp: GoogleBooksResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let vol = resp
        .items
        .as_ref()
        .and_then(|items| items.first())
        .map(|item| &item.volume_info)
        .ok_or("No results from Google Books")?;

    let isbn = vol
        .industry_identifiers
        .as_ref()
        .and_then(|ids| {
            ids.iter()
                .find(|id| id.id_type == "ISBN_13" || id.id_type == "ISBN_10")
                .map(|id| id.identifier.clone())
        });

    Ok(BookMetadata {
        title: vol.title.clone(),
        author: vol.authors.as_ref().and_then(|a| a.first().cloned()),
        cover_url: vol
            .image_links
            .as_ref()
            .and_then(|il| il.thumbnail.clone()),
        description: vol.description.clone(),
        publisher: vol.publisher.clone(),
        published_date: vol.published_date.clone(),
        page_count: vol.page_count,
        isbn,
    })
}

async fn google_books(isbn: &str) -> Result<BookMetadata, String> {
    let url = format!(
        "https://www.googleapis.com/books/v1/volumes?q=isbn:{}",
        isbn
    );
    let client = reqwest::Client::new();
    let resp: GoogleBooksResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let vol = resp
        .items
        .as_ref()
        .and_then(|items| items.first())
        .map(|item| &item.volume_info)
        .ok_or("ISBN not found on Google Books")?;

    let isbn = vol
        .industry_identifiers
        .as_ref()
        .and_then(|ids| {
            ids.iter()
                .find(|id| id.id_type == "ISBN_13" || id.id_type == "ISBN_10")
                .map(|id| id.identifier.clone())
        });

    Ok(BookMetadata {
        title: vol.title.clone(),
        author: vol.authors.as_ref().and_then(|a| a.first().cloned()),
        cover_url: vol
            .image_links
            .as_ref()
            .and_then(|il| il.thumbnail.clone()),
        description: vol.description.clone(),
        publisher: vol.publisher.clone(),
        published_date: vol.published_date.clone(),
        page_count: vol.page_count,
        isbn,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_isbn_with_dashes() {
        assert_eq!(sanitize_isbn("978-0-14-143951-8"), "9780141439518");
    }

    #[test]
    fn sanitize_isbn_with_spaces() {
        assert_eq!(sanitize_isbn("978 0141439518"), "9780141439518");
    }

    #[test]
    fn sanitize_isbn_empty() {
        assert_eq!(sanitize_isbn(""), "");
    }

    #[test]
    fn sanitize_isbn_already_clean() {
        assert_eq!(sanitize_isbn("9780141439518"), "9780141439518");
    }
}
