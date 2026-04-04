use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub page_count: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
    pub rating: Option<i32>,
    pub status: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryEntry {
    pub id: i64,
    pub book_id: i64,
    pub book_title: String,
    pub book_author: Option<String>,
    pub book_cover_url: Option<String>,
    pub body: Option<String>,
    pub rating: Option<i32>,
    pub entry_date: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shelf {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub id: i64,
    pub book_id: i64,
    pub text: String,
    pub location_start: Option<i64>,
    pub location_end: Option<i64>,
    pub page: Option<i64>,
    pub clip_type: String,
    pub clipped_at: Option<String>,
    pub created_at: String,
}
