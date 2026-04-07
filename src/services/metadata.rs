use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{info, instrument, warn};

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Blurb/0.1.0 (book library app)")
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
});

static OPEN_LIBRARY_LIMITER: LazyLock<Mutex<Instant>> =
    LazyLock::new(|| Mutex::new(Instant::now()));
static GOOGLE_BOOKS_LIMITER: LazyLock<Mutex<Instant>> =
    LazyLock::new(|| Mutex::new(Instant::now()));

const OPEN_LIBRARY_INTERVAL: Duration = Duration::from_millis(1000);
const GOOGLE_BOOKS_INTERVAL: Duration = Duration::from_millis(1500);
const MAX_RETRIES: u32 = 3;

async fn rate_limit(limiter: &Mutex<Instant>, min_interval: Duration) {
    let mut last = limiter.lock().await;
    let elapsed = last.elapsed();
    if elapsed < min_interval {
        tokio::time::sleep(min_interval - elapsed).await;
    }
    *last = Instant::now();
}

async fn send_with_backoff(
    client: &reqwest::Client,
    url: &str,
    limiter: &Mutex<Instant>,
    min_interval: Duration,
) -> Result<reqwest::Response, String> {
    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            rate_limit(limiter, min_interval).await;
        }
        let resp = client.get(url).send().await.map_err(|e| e.to_string())?;

        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            if attempt == MAX_RETRIES {
                return Err("429 Too Many Requests after max retries".into());
            }
            let backoff = resp
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(1 << attempt));
            warn!(url, attempt, "429 rate limited, backing off {:?}", backoff);
            tokio::time::sleep(backoff).await;
            continue;
        }

        if attempt > 0 {
            info!(url, attempt, "request succeeded after retry");
        }
        return resp.error_for_status().map_err(|e| e.to_string());
    }
    unreachable!()
}

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

#[instrument(skip_all, fields(isbn = %isbn), err(level = tracing::Level::WARN))]
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
    rate_limit(&OPEN_LIBRARY_LIMITER, OPEN_LIBRARY_INTERVAL).await;
    let resp: std::collections::HashMap<String, OLBookData> =
        send_with_backoff(&client, &url, &OPEN_LIBRARY_LIMITER, OPEN_LIBRARY_INTERVAL)
            .await?
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

fn prefer_isbn13(isbns: Option<Vec<String>>) -> Option<String> {
    let isbns = isbns?;
    isbns
        .iter()
        .find(|i| i.starts_with("978") || i.starts_with("979"))
        .cloned()
        .or_else(|| isbns.into_iter().next())
}

#[instrument(skip_all, err(level = tracing::Level::WARN))]
pub async fn search_covers(query: &str) -> Result<Vec<BookMetadata>, String> {
    let client = HTTP_CLIENT.clone();
    let url = format!(
        "https://openlibrary.org/search.json?q={}&fields=title,author_name,cover_i,isbn,publisher,first_publish_year,number_of_pages_median,key&limit=5",
        urlencoding::encode(query)
    );
    rate_limit(&OPEN_LIBRARY_LIMITER, OPEN_LIBRARY_INTERVAL).await;
    let resp: OLSearchResponse =
        send_with_backoff(&client, &url, &OPEN_LIBRARY_LIMITER, OPEN_LIBRARY_INTERVAL)
            .await?
            .json()
            .await
            .map_err(|e| e.to_string())?;

    let mut seen_keys = std::collections::HashSet::new();
    let results = resp
        .docs
        .unwrap_or_default()
        .into_iter()
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

pub fn sanitize_title(title: &str) -> String {
    static RE_EDITION_PARENS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)\s*\([^)]*(?:Edition|Classics|International|Reprint|Paperback|Hardcover)[^)]*\)",
        )
        .unwrap()
    });
    static RE_SERIES_PARENS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\s*\([^)]*(?:Book|Series)\s+\d+[^)]*\)").unwrap());
    static RE_SUBTITLE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i):?\s*A\s+(?:Novel|Memoir|Story|Thriller)\s*$").unwrap());
    static RE_MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());

    let s = RE_EDITION_PARENS.replace_all(title, "");
    let s = RE_SERIES_PARENS.replace_all(&s, "");
    let s = RE_SUBTITLE.replace_all(&s, "");
    let s = RE_MULTI_SPACE.replace_all(&s, " ");
    s.trim().to_string()
}

pub fn validate_match(
    original_title: &str,
    original_author: Option<&str>,
    result: &BookMetadata,
) -> bool {
    fn normalize(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    const STOP_WORDS: &[&str] = &["the", "a", "an", "of", "in", "on", "and", "or"];

    let core = original_title.split(':').next().unwrap_or(original_title);
    let core_norm = normalize(core);
    let sig_words: Vec<&str> = core_norm
        .split_whitespace()
        .filter(|w| !STOP_WORDS.contains(w))
        .collect();

    let result_title_norm = result.title.as_deref().map(normalize).unwrap_or_default();

    let title_ok = if sig_words.is_empty() {
        // Short/punctuation-only titles: exact normalized match
        result_title_norm.contains(&core_norm)
    } else {
        let matched = sig_words
            .iter()
            .filter(|w| result_title_norm.contains(**w))
            .count();
        matched * 2 >= sig_words.len() // at least 50%
    };

    let author_ok = match original_author {
        Some(auth) => {
            let last_word = auth.split_whitespace().last().unwrap_or(auth);
            match result.author.as_deref() {
                Some(result_auth) => result_auth
                    .to_lowercase()
                    .contains(&last_word.to_lowercase()),
                None => true, // no result author to check against
            }
        }
        None => true,
    };

    title_ok && author_ok
}

#[instrument(skip_all, err(level = tracing::Level::WARN))]
pub async fn search_by_title(title: &str, author: Option<&str>) -> Result<BookMetadata, String> {
    let clean_title = sanitize_title(title);

    if let Ok(meta) = search_by_title_open_library(&clean_title, author).await {
        if meta.title.is_some() && validate_match(title, author, &meta) {
            info!(
                title,
                result_title = meta.title.as_deref(),
                "matched via Open Library"
            );
            return Ok(meta);
        }
    }

    if let Ok(meta) = search_by_title_google(&clean_title, author).await {
        if validate_match(title, author, &meta) {
            info!(
                title,
                result_title = meta.title.as_deref(),
                "matched via Google Books"
            );
            return Ok(meta);
        }
    }

    warn!(title, "no confident match found from any source");
    Err("no confident match found".into())
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
    rate_limit(&OPEN_LIBRARY_LIMITER, OPEN_LIBRARY_INTERVAL).await;
    let resp: OLSearchResponse =
        send_with_backoff(&client, &url, &OPEN_LIBRARY_LIMITER, OPEN_LIBRARY_INTERVAL)
            .await?
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
    rate_limit(&GOOGLE_BOOKS_LIMITER, GOOGLE_BOOKS_INTERVAL).await;
    let resp: GoogleBooksResponse =
        send_with_backoff(&client, &url, &GOOGLE_BOOKS_LIMITER, GOOGLE_BOOKS_INTERVAL)
            .await?
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
    rate_limit(&GOOGLE_BOOKS_LIMITER, GOOGLE_BOOKS_INTERVAL).await;
    let resp: GoogleBooksResponse =
        send_with_backoff(&client, &url, &GOOGLE_BOOKS_LIMITER, GOOGLE_BOOKS_INTERVAL)
            .await?
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
