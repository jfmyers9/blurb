use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct LetterboxdEntry {
    pub title: String,
    pub year: Option<i32>,
    pub rating: Option<f64>,
    pub rating_int: Option<i32>,
    pub watched_date: Option<String>,
    pub rewatch: bool,
    pub tags: Vec<String>,
    pub letterboxd_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiaryRow {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "Letterboxd URI")]
    letterboxd_uri: Option<String>,
    #[serde(rename = "Rating")]
    rating: Option<String>,
    #[serde(rename = "Rewatch")]
    rewatch: Option<String>,
    #[serde(rename = "Tags")]
    tags: Option<String>,
    #[serde(rename = "Watched Date")]
    watched_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RatingsRow {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "Letterboxd URI")]
    letterboxd_uri: Option<String>,
    #[serde(rename = "Rating")]
    rating: Option<String>,
}

/// Convert Letterboxd 0.5-5.0 scale to 1-5 integer scale.
pub fn convert_rating(r: f64) -> i32 {
    // 0.5→1, 1.0→1, 1.5→2, 2.0→2, 2.5→3, 3.0→3, 3.5→4, 4.0→4, 4.5→5, 5.0→5
    (r.ceil()) as i32
}

fn parse_rating(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed
        .parse::<f64>()
        .ok()
        .filter(|v| (0.5..=5.0).contains(v))
}

fn parse_year(s: &str) -> Option<i32> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<i32>().ok()
}

fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn parse_diary_csv(content: &str) -> Result<Vec<LetterboxdEntry>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut entries = Vec::new();
    for (i, result) in rdr.deserialize().enumerate() {
        let row: DiaryRow = result.map_err(|e| format!("Row {}: {}", i + 1, e))?;

        let rating = row.rating.as_deref().and_then(parse_rating);
        let rating_int = rating.map(convert_rating);
        let rewatch = row
            .rewatch
            .as_deref()
            .map(|s| s.trim().eq_ignore_ascii_case("yes"))
            .unwrap_or(false);
        let tags: Vec<String> = row
            .tags
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter_map(non_empty)
            .collect();

        entries.push(LetterboxdEntry {
            title: row.name.trim().to_string(),
            year: row.year.as_deref().and_then(parse_year),
            rating,
            rating_int,
            watched_date: row.watched_date.as_deref().and_then(non_empty),
            rewatch,
            tags,
            letterboxd_uri: row.letterboxd_uri.as_deref().and_then(non_empty),
        });
    }
    Ok(entries)
}

pub fn parse_ratings_csv(content: &str) -> Result<Vec<LetterboxdEntry>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut entries = Vec::new();
    for (i, result) in rdr.deserialize().enumerate() {
        let row: RatingsRow = result.map_err(|e| format!("Row {}: {}", i + 1, e))?;

        let rating = row.rating.as_deref().and_then(parse_rating);
        let rating_int = rating.map(convert_rating);

        entries.push(LetterboxdEntry {
            title: row.name.trim().to_string(),
            year: row.year.as_deref().and_then(parse_year),
            rating,
            rating_int,
            watched_date: None,
            rewatch: false,
            tags: Vec::new(),
            letterboxd_uri: row.letterboxd_uri.as_deref().and_then(non_empty),
        });
    }
    Ok(entries)
}

/// Merge diary and ratings entries. Diary entries take precedence.
/// For movies that appear only in ratings, the rating-only entry is kept.
pub fn merge_entries(
    diary: Vec<LetterboxdEntry>,
    ratings: Vec<LetterboxdEntry>,
) -> Vec<LetterboxdEntry> {
    // Key: (lowercase title, year)
    let mut merged: HashMap<(String, Option<i32>), LetterboxdEntry> = HashMap::new();

    // Insert ratings first (lower priority)
    for entry in ratings {
        let key = (entry.title.to_lowercase(), entry.year);
        merged.insert(key, entry);
    }

    // Diary entries override ratings
    for entry in diary {
        let key = (entry.title.to_lowercase(), entry.year);
        let existing_rating = merged.get(&key).and_then(|e| e.rating);

        // If diary entry has no rating but ratings entry does, keep the rating
        let final_entry = if entry.rating.is_none() && existing_rating.is_some() {
            let rating = existing_rating;
            let rating_int = rating.map(convert_rating);
            LetterboxdEntry {
                rating,
                rating_int,
                ..entry
            }
        } else {
            entry
        };

        merged.insert(key, final_entry);
    }

    let mut result: Vec<LetterboxdEntry> = merged.into_values().collect();
    result.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    result
}

#[cfg(test)]
#[path = "letterboxd_tests.rs"]
mod tests;
