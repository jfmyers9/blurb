use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
});

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
    let client = HTTP_CLIENT.clone();
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
    isbn: Option<Vec<String>>,
    key: Option<String>,
    publisher: Option<Vec<String>>,
    first_publish_year: Option<i32>,
    number_of_pages_median: Option<i32>,
}

/// Prefer ISBN-13 (978/979 prefix), falling back to first available.
fn prefer_isbn13(isbns: Option<Vec<String>>) -> Option<String> {
    let isbns = isbns?;
    isbns
        .iter()
        .find(|i| i.starts_with("978") || i.starts_with("979"))
        .cloned()
        .or_else(|| isbns.into_iter().next())
}

pub async fn search_covers(query: &str) -> Result<Vec<BookMetadata>, String> {
    let client = HTTP_CLIENT.clone();
    let url = format!(
        "https://openlibrary.org/search.json?q={}&fields=title,author_name,cover_i,isbn,publisher,first_publish_year,number_of_pages_median,key&limit=5",
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

    let mut seen_keys = std::collections::HashSet::new();
    let results = resp
        .docs
        .unwrap_or_default()
        .into_iter()
        // Dedup by `key`; docs without a key always pass through since they can't be matched.
        .filter(|doc| doc.key.as_ref().is_none_or(|k| seen_keys.insert(k.clone())))
        .map(|doc| BookMetadata {
            title: doc.title,
            author: doc.author_name.as_ref().and_then(|a| a.first().cloned()),
            cover_url: doc
                .cover_i
                .map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id)),
            description: None,
            publisher: doc.publisher.and_then(|p| p.into_iter().next()),
            published_date: doc.first_publish_year.map(|y| y.to_string()),
            page_count: doc.number_of_pages_median,
            isbn: prefer_isbn13(doc.isbn),
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
        "https://openlibrary.org/search.json?q={}&fields=title,author_name,cover_i,isbn,key,publisher,first_publish_year,number_of_pages_median&limit=1",
        urlencoding::encode(&query)
    );
    let client = HTTP_CLIENT.clone();
    let resp: OLSearchResponse = client
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

    let isbn = prefer_isbn13(doc.isbn);
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

async fn search_by_title_google(title: &str, author: Option<&str>) -> Result<BookMetadata, String> {
    let query = match author {
        Some(a) => format!(
            "intitle:{}+inauthor:{}",
            urlencoding::encode(title),
            urlencoding::encode(a)
        ),
        None => format!("intitle:{}", urlencoding::encode(title)),
    };
    let url = format!("https://www.googleapis.com/books/v1/volumes?q={}", query);
    let client = HTTP_CLIENT.clone();
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

    let isbn = vol.industry_identifiers.as_ref().and_then(|ids| {
        ids.iter()
            .find(|id| id.id_type == "ISBN_13" || id.id_type == "ISBN_10")
            .map(|id| id.identifier.clone())
    });

    Ok(BookMetadata {
        title: vol.title.clone(),
        author: vol.authors.as_ref().and_then(|a| a.first().cloned()),
        cover_url: vol.image_links.as_ref().and_then(|il| il.thumbnail.clone()),
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
    let client = HTTP_CLIENT.clone();
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

    let isbn = vol.industry_identifiers.as_ref().and_then(|ids| {
        ids.iter()
            .find(|id| id.id_type == "ISBN_13" || id.id_type == "ISBN_10")
            .map(|id| id.identifier.clone())
    });

    Ok(BookMetadata {
        title: vol.title.clone(),
        author: vol.authors.as_ref().and_then(|a| a.first().cloned()),
        cover_url: vol.image_links.as_ref().and_then(|il| il.thumbnail.clone()),
        description: vol.description.clone(),
        publisher: vol.publisher.clone(),
        published_date: vol.published_date.clone(),
        page_count: vol.page_count,
        isbn,
    })
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
