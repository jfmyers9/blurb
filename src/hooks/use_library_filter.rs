use std::collections::HashMap;

use dioxus::prelude::*;

use crate::data::models::{Book, Shelf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOption {
    DateAdded,
    Title,
    Author,
    Rating,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Grid,
    List,
}

pub struct LibraryFilter {
    pub search_query: Signal<String>,
    pub active_status: Signal<String>,
    pub sort_by: Signal<SortOption>,
    pub active_shelf: Signal<Option<i64>>,
    pub view_mode: Signal<ViewMode>,
    pub min_rating: Signal<Option<i32>>,
    pub filtered_books: Memo<Vec<Book>>,
}

pub fn use_library_filter(
    books: Signal<Vec<Book>>,
    shelves: Signal<Vec<Shelf>>,
    shelf_book_ids: Signal<HashMap<i64, Vec<i64>>>,
) -> LibraryFilter {
    let search_query = use_signal(String::new);
    let active_status = use_signal(|| "all".to_string());
    let sort_by = use_signal(|| SortOption::DateAdded);
    let active_shelf: Signal<Option<i64>> = use_signal(|| None);
    let view_mode = use_signal(|| ViewMode::Grid);
    let min_rating: Signal<Option<i32>> = use_signal(|| None);

    let filtered_books = use_memo(move || {
        let all_books = books.read();
        let status = active_status.read();
        let shelf = *active_shelf.read();
        let sort = *sort_by.read();
        let query = search_query.read().to_lowercase();
        let rating_min = *min_rating.read();
        let shelf_map = shelf_book_ids.read();
        let shelves_list = shelves.read();

        let mut filtered: Vec<Book> = all_books
            .iter()
            .filter(|b| {
                if *status != "all" && b.status.as_deref() != Some(status.as_str()) {
                    return false;
                }
                if let Some(sid) = shelf {
                    let ids = shelf_map.get(&sid).cloned().unwrap_or_default();
                    if !ids.contains(&b.id) {
                        return false;
                    }
                }
                if let Some(min) = rating_min {
                    if b.rating.unwrap_or(0) < min {
                        return false;
                    }
                }
                if !query.is_empty() {
                    let shelf_names: Vec<String> = shelves_list
                        .iter()
                        .filter(|s| {
                            shelf_map
                                .get(&s.id)
                                .map(|ids| ids.contains(&b.id))
                                .unwrap_or(false)
                        })
                        .map(|s| s.name.clone())
                        .collect();
                    let searchable = format!(
                        "{} {} {} {}",
                        b.title,
                        b.author.as_deref().unwrap_or(""),
                        b.isbn.as_deref().unwrap_or(""),
                        shelf_names.join(" ")
                    )
                    .to_lowercase();
                    if !searchable.contains(&query) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        filtered.sort_by(|a, b| match sort {
            SortOption::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            SortOption::Author => {
                let aa = a.author.as_deref().unwrap_or("");
                let bb = b.author.as_deref().unwrap_or("");
                aa.to_lowercase().cmp(&bb.to_lowercase())
            }
            SortOption::Rating => b.rating.unwrap_or(0).cmp(&a.rating.unwrap_or(0)),
            SortOption::DateAdded => b.created_at.cmp(&a.created_at),
        });

        filtered
    });

    LibraryFilter {
        search_query,
        active_status,
        sort_by,
        active_shelf,
        view_mode,
        min_rating,
        filtered_books,
    }
}
