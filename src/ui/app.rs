use std::collections::HashMap;

use dioxus::prelude::*;

use tracing::error;

use crate::data::commands::{list_all_shelf_book_ids_db, list_books_db, list_shelves_db};
use crate::data::models::{Book, DiaryEntry, Shelf};
use crate::hooks::{use_library_filter, ViewMode};
use crate::DatabaseHandle;

use super::add_book_form::AddBookForm;
use super::book_detail::BookDetail;
use super::command_palette::CommandPalette;
use super::diary_entry_form::DiaryEntryForm;
use super::diary_feed::DiaryFeed;
use super::enrichment_bar::EnrichmentBar;
use super::goodreads_import::GoodreadsImport;
use super::kindle_sync::KindleSync;
use super::library_grid::LibraryGrid;
use super::library_list::LibraryList;
use super::status_filter_bar::StatusFilterBar;
use crate::services::enrichment::EnrichmentState;

#[derive(Clone, Copy, PartialEq)]
enum AppView {
    Library,
    Diary,
}

#[component]
pub fn App() -> Element {
    use_context_provider(EnrichmentState::new);
    let db = use_context::<DatabaseHandle>();

    let mut books: Signal<Vec<Book>> = use_signal(Vec::new);
    let mut shelves: Signal<Vec<Shelf>> = use_signal(Vec::new);
    let mut shelf_book_ids: Signal<HashMap<i64, Vec<i64>>> = use_signal(HashMap::new);

    let mut selected_book_id: Signal<Option<i64>> = use_signal(|| None);
    let mut selected_diary_entry: Signal<Option<DiaryEntry>> = use_signal(|| None);
    let mut show_add_form = use_signal(|| false);
    let mut show_kindle_sync = use_signal(|| false);
    let mut show_goodreads_import = use_signal(|| false);
    let mut palette_open = use_signal(|| false);
    let mut current_view = use_signal(|| AppView::Library);

    let reload_data = {
        let db = db.clone();
        move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                match list_books_db(&conn) {
                    Ok(b) => books.set(b),
                    Err(e) => error!("failed to load books: {e}"),
                }
                match list_shelves_db(&conn) {
                    Ok(s) => shelves.set(s),
                    Err(e) => error!("failed to load shelves: {e}"),
                }
                match list_all_shelf_book_ids_db(&conn) {
                    Ok(pairs) => {
                        let mut map: HashMap<i64, Vec<i64>> = HashMap::new();
                        for (shelf_id, book_id) in pairs {
                            map.entry(shelf_id).or_default().push(book_id);
                        }
                        shelf_book_ids.set(map);
                    }
                    Err(e) => error!("failed to load shelf-book mappings: {e}"),
                }
            });
        }
    };

    // Load data on mount
    {
        let reload_data = reload_data.clone();
        use_effect(move || {
            reload_data();
        });
    }

    let mut filter = use_library_filter(books, shelves, shelf_book_ids);

    let shelf_book_counts: HashMap<i64, usize> = {
        let map = shelf_book_ids.read();
        map.iter().map(|(k, v)| (*k, v.len())).collect()
    };

    rsx! {
        link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        link { rel: "stylesheet", href: asset!("/assets/editor.css") }
        script { src: asset!("/assets/tiptap-bundle.js") }

        div {
            class: "flex min-h-screen flex-col bg-gray-50 dark:bg-gray-950",
            // Cmd+K handler
            onkeydown: move |e: KeyboardEvent| {
                if e.modifiers().contains(Modifiers::META) && e.key() == Key::Character("k".to_string()) {
                    let current = *palette_open.read();
                    palette_open.set(!current);
                } else if e.key() == Key::Escape {
                    let current = *palette_open.read();
                    if current {
                        palette_open.set(false);
                    }
                }
            },
            tabindex: "0",

            // Top bar
            header {
                class: "sticky top-0 z-30 flex items-center justify-between
                    border-b border-gray-200 bg-white/80 px-6 py-3 backdrop-blur
                    dark:border-gray-800 dark:bg-gray-900/80",
                div {
                    class: "flex items-center gap-4",
                    h1 {
                        class: "text-xl font-bold tracking-tight text-gray-900 dark:text-gray-100",
                        "Blurb"
                    }
                    // View tabs
                    nav {
                        class: "flex gap-1",
                        button {
                            r#type: "button",
                            onclick: move |_| current_view.set(AppView::Library),
                            class: if *current_view.read() == AppView::Library {
                                "rounded-md px-3 py-1 text-sm font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                            } else {
                                "rounded-md px-3 py-1 text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                            },
                            "Library"
                        }
                        button {
                            r#type: "button",
                            onclick: move |_| current_view.set(AppView::Diary),
                            class: if *current_view.read() == AppView::Diary {
                                "rounded-md px-3 py-1 text-sm font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                            } else {
                                "rounded-md px-3 py-1 text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                            },
                            "Diary"
                        }
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    // Cmd+K button
                    button {
                        r#type: "button",
                        title: "Search (Cmd+K)",
                        onclick: move |_| palette_open.set(true),
                        class: "flex h-9 items-center gap-1.5 rounded-lg border border-gray-200
                            bg-white px-3 text-sm text-gray-400 shadow-sm transition
                            hover:border-gray-300 dark:border-gray-700 dark:bg-gray-800
                            dark:text-gray-500 dark:hover:border-gray-600",
                        svg {
                            class: "h-4 w-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
                            }
                        }
                        span { class: "hidden sm:inline", "Search..." }
                        kbd {
                            class: "ml-2 hidden rounded bg-gray-100 px-1.5 py-0.5 text-xs
                                font-medium text-gray-500 sm:inline dark:bg-gray-700 dark:text-gray-400",
                            "\u{2318}K"
                        }
                    }
                    // Kindle sync button
                    button {
                        r#type: "button",
                        title: "Kindle Sync",
                        onclick: move |_| show_kindle_sync.set(true),
                        class: "flex h-9 w-9 items-center justify-center rounded-full
                            text-gray-400 transition hover:bg-gray-100 hover:text-gray-600
                            dark:hover:bg-gray-800 dark:hover:text-gray-300",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "1.5",
                                d: "M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25",
                            }
                        }
                    }
                    // Goodreads import button
                    button {
                        r#type: "button",
                        title: "Import Goodreads",
                        onclick: move |_| show_goodreads_import.set(true),
                        class: "flex h-9 w-9 items-center justify-center rounded-full
                            text-gray-400 transition hover:bg-gray-100 hover:text-gray-600
                            dark:hover:bg-gray-800 dark:hover:text-gray-300",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "1.5",
                                d: "M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5",
                            }
                        }
                    }
                    // Add book button
                    button {
                        r#type: "button",
                        title: "Add book",
                        onclick: move |_| show_add_form.set(true),
                        class: "flex h-9 w-9 items-center justify-center rounded-full
                            bg-amber-600 text-white shadow-sm transition hover:bg-amber-700
                            active:scale-95",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 4v16m8-8H4",
                            }
                        }
                    }
                }
            }

            // Main content
            main {
                class: "flex-1",
                match *current_view.read() {
                    AppView::Library => rsx! {
                        StatusFilterBar {
                            books: filter.filtered_books.read().clone(),
                            active_status: filter.active_status.read().clone(),
                            on_status_change: move |s: String| filter.active_status.set(s),
                            sort_by: *filter.sort_by.read(),
                            on_sort_change: move |s| filter.sort_by.set(s),
                            shelves: shelves.read().clone(),
                            active_shelf: *filter.active_shelf.read(),
                            on_shelf_change: move |s| filter.active_shelf.set(s),
                            shelf_book_counts: shelf_book_counts.clone(),
                            search_query: filter.search_query.read().clone(),
                            on_search_change: move |q: String| filter.search_query.set(q),
                            view_mode: *filter.view_mode.read(),
                            on_view_mode_change: move |m| filter.view_mode.set(m),
                            min_rating: *filter.min_rating.read(),
                            on_min_rating_change: move |r| filter.min_rating.set(r),
                            on_clear_all: move |_| {
                                filter.active_status.set("all".to_string());
                                filter.min_rating.set(None);
                                filter.active_shelf.set(None);
                                filter.search_query.set(String::new());
                            },
                        }

                        {
                            let filtered = filter.filtered_books.read();
                            if filtered.is_empty() {
                                rsx! {
                                    div {
                                        class: "flex flex-1 flex-col items-center justify-center py-24 text-center",
                                        div {
                                            class: "mb-4 text-6xl opacity-30",
                                            "\u{1f4da}"
                                        }
                                        h2 {
                                            class: "text-lg font-medium text-gray-600 dark:text-gray-400",
                                            "Your library is empty"
                                        }
                                        p {
                                            class: "mt-1 text-sm text-gray-400 dark:text-gray-500",
                                            "Add your first book with the + button above."
                                        }
                                    }
                                }
                            } else if *filter.view_mode.read() == ViewMode::Grid {
                                rsx! {
                                    LibraryGrid {
                                        books: filtered.clone(),
                                        on_select_book: move |id: i64| selected_book_id.set(Some(id)),
                                    }
                                }
                            } else {
                                rsx! {
                                    LibraryList {
                                        books: filtered.clone(),
                                        on_select_book: move |id: i64| selected_book_id.set(Some(id)),
                                    }
                                }
                            }
                        }
                    },
                    AppView::Diary => rsx! {
                        DiaryFeed {
                            on_select_entry: move |entry: DiaryEntry| selected_diary_entry.set(Some(entry)),
                        }
                    },
                }
            }
        }

        EnrichmentBar {}

        // Book detail slide-out panel
        if let Some(bid) = *selected_book_id.read() {
            BookDetail {
                key: "{bid}",
                book_id: bid,
                shelves: shelves.read().clone(),
                on_close: move |_| selected_book_id.set(None),
                on_changed: {
                    let reload_data = reload_data.clone();
                    move |_| reload_data()
                },
                on_deleted: {
                    let reload_data = reload_data.clone();
                    move |_id: i64| {
                        selected_book_id.set(None);
                        reload_data();
                    }
                },
            }
        }

        // Diary entry detail overlay
        if let Some(ref entry) = *selected_diary_entry.read() {
            DiaryEntryForm {
                book_id: entry.book_id,
                book_title: Some(entry.book_title.clone()),
                entry: Some(entry.clone()),
                on_save: move |_| {
                    selected_diary_entry.set(None);
                },
                on_close: move |_| selected_diary_entry.set(None),
            }
        }

        // Add book modal
        if *show_add_form.read() {
            AddBookForm {
                on_close: move |_| show_add_form.set(false),
                on_added: {
                    let reload_data = reload_data.clone();
                    move |_id: i64| {
                        reload_data();
                    }
                },
            }
        }

        // Kindle sync modal
        if *show_kindle_sync.read() {
            KindleSync {
                on_close: move |_| show_kindle_sync.set(false),
                on_import_complete: {
                    let reload_data = reload_data.clone();
                    move |_| reload_data()
                },
            }
        }

        // Goodreads import modal
        if *show_goodreads_import.read() {
            GoodreadsImport {
                on_close: move |_| show_goodreads_import.set(false),
                on_import_complete: {
                    let reload_data = reload_data.clone();
                    move |_| reload_data()
                },
            }
        }

        // Command palette overlay
        CommandPalette {
            is_open: *palette_open.read(),
            on_close: move |_| palette_open.set(false),
            books: books.read().clone(),
            on_select_book: move |id: i64| {
                selected_book_id.set(Some(id));
            },
            on_command: move |cmd: String| {
                match cmd.as_str() {
                    "add-book" => show_add_form.set(true),
                    "switch-library" => current_view.set(AppView::Library),
                    "switch-diary" => current_view.set(AppView::Diary),
                    "toggle-view" => {
                        let current = *filter.view_mode.read();
                        filter.view_mode.set(if current == ViewMode::Grid {
                            ViewMode::List
                        } else {
                            ViewMode::Grid
                        });
                    }
                    "kindle-sync" => show_kindle_sync.set(true),
                    "goodreads-import" => show_goodreads_import.set(true),
                    _ => {}
                }
            },
        }
    }
}
