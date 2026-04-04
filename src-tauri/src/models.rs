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
    pub review: Option<String>,
}
