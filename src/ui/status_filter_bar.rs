use std::collections::HashMap;

use dioxus::prelude::*;

use crate::data::models::{Book, Shelf};
use crate::hooks::{SortOption, ViewMode};

use super::sort_dropdown::SortDropdown;

const FILTER_STATUSES: &[(&str, &str)] = &[
    ("all", "All"),
    ("want_to_read", "Want to Read"),
    ("reading", "Reading"),
    ("finished", "Finished"),
    ("abandoned", "Abandoned"),
];

fn status_color(status: &str) -> &'static str {
    match status {
        "want_to_read" => "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300",
        "reading" => "bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300",
        "finished" => "bg-purple-100 text-purple-800 dark:bg-purple-900/40 dark:text-purple-300",
        "abandoned" => "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
        _ => "",
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct StatusFilterBarProps {
    books: Vec<Book>,
    active_status: String,
    on_status_change: EventHandler<String>,
    sort_by: SortOption,
    on_sort_change: EventHandler<SortOption>,
    shelves: Vec<Shelf>,
    active_shelf: Option<i64>,
    on_shelf_change: EventHandler<Option<i64>>,
    shelf_book_counts: HashMap<i64, usize>,
    search_query: String,
    on_search_change: EventHandler<String>,
    view_mode: ViewMode,
    on_view_mode_change: EventHandler<ViewMode>,
    min_rating: Option<i32>,
    on_min_rating_change: EventHandler<Option<i32>>,
    on_clear_all: EventHandler<()>,
}

#[component]
pub fn StatusFilterBar(props: StatusFilterBarProps) -> Element {
    let counts = {
        let mut m: HashMap<String, usize> = HashMap::new();
        m.insert("all".to_string(), props.books.len());
        for book in &props.books {
            if let Some(ref s) = book.status {
                if !s.is_empty() {
                    *m.entry(s.clone()).or_insert(0) += 1;
                }
            }
        }
        m
    };

    let has_ratings = props.books.iter().any(|b| b.rating.is_some());
    let has_shelves = !props.shelves.is_empty();
    let has_active_filters = props.active_status != "all"
        || props.min_rating.is_some()
        || props.active_shelf.is_some()
        || !props.search_query.is_empty();

    rsx! {
        div {
            class: "sticky top-[49px] z-20 space-y-2 px-6 pt-5 pb-1 bg-white/80 backdrop-blur dark:bg-gray-900/80",

            // Status row
            div {
                class: "flex items-center justify-between gap-4",
                div {
                    class: "flex flex-wrap items-center gap-1.5",
                    span {
                        class: "text-[10px] font-medium uppercase tracking-wider text-gray-400",
                        "Status"
                    }
                    for &(value, label) in FILTER_STATUSES.iter() {
                        {
                            let is_active = props.active_status == value;
                            let count = counts.get(value).copied().unwrap_or(0);
                            let active_class = if is_active {
                                if value != "all" {
                                    status_color(value)
                                } else {
                                    "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                                }
                            } else {
                                "bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                            };
                            let count_class = if is_active {
                                "bg-white/40 dark:bg-black/20"
                            } else {
                                "bg-gray-200/80 dark:bg-gray-700"
                            };
                            let val = value.to_string();
                            rsx! {
                                button {
                                    key: "{value}",
                                    r#type: "button",
                                    onclick: move |_| props.on_status_change.call(val.clone()),
                                    class: "inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium active:scale-95 transition-all duration-150 {active_class}",
                                    "{label}"
                                    span {
                                        class: "inline-flex h-4.5 min-w-[1.125rem] items-center justify-center rounded-full px-1 text-[10px] font-semibold leading-none {count_class}",
                                        "{count}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Search
                div {
                    class: "relative",
                    svg {
                        class: "pointer-events-none absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-gray-400",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z",
                        }
                    }
                    input {
                        r#type: "text",
                        value: "{props.search_query}",
                        oninput: move |evt| props.on_search_change.call(evt.value()),
                        placeholder: "Search...",
                        class: "w-48 focus:w-64 transition-all duration-200 rounded-md border border-gray-300 bg-white py-1.5 pl-7 pr-2.5 text-xs
                            text-gray-700 placeholder-gray-400 dark:border-gray-600 dark:bg-gray-800
                            dark:text-gray-300 dark:placeholder-gray-500
                            focus:ring-2 focus:ring-amber-500 focus:outline-none",
                    }
                }

                // Sort
                SortDropdown {
                    value: props.sort_by,
                    on_change: move |v| props.on_sort_change.call(v),
                }

                // View mode toggle
                div {
                    class: "flex gap-0.5",
                    button {
                        r#type: "button",
                        title: "Grid view",
                        onclick: move |_| props.on_view_mode_change.call(ViewMode::Grid),
                        class: if props.view_mode == ViewMode::Grid {
                            "flex h-8 w-8 items-center justify-center rounded-md transition-colors bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                        } else {
                            "flex h-8 w-8 items-center justify-center rounded-md transition-colors text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
                        },
                        svg {
                            class: "h-4 w-4",
                            fill: "currentColor",
                            view_box: "0 0 16 16",
                            rect { x: "1", y: "1", width: "6", height: "6", rx: "1" }
                            rect { x: "9", y: "1", width: "6", height: "6", rx: "1" }
                            rect { x: "1", y: "9", width: "6", height: "6", rx: "1" }
                            rect { x: "9", y: "9", width: "6", height: "6", rx: "1" }
                        }
                    }
                    button {
                        r#type: "button",
                        title: "List view",
                        onclick: move |_| props.on_view_mode_change.call(ViewMode::List),
                        class: if props.view_mode == ViewMode::List {
                            "flex h-8 w-8 items-center justify-center rounded-md transition-colors bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                        } else {
                            "flex h-8 w-8 items-center justify-center rounded-md transition-colors text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
                        },
                        svg {
                            class: "h-4 w-4",
                            fill: "currentColor",
                            view_box: "0 0 16 16",
                            rect { x: "1", y: "2", width: "14", height: "2", rx: "0.5" }
                            rect { x: "1", y: "7", width: "14", height: "2", rx: "0.5" }
                            rect { x: "1", y: "12", width: "14", height: "2", rx: "0.5" }
                        }
                    }
                }
            }

            // Rating row
            if has_ratings {
                div {
                    class: "flex flex-wrap items-center gap-1.5 border-t border-gray-200 pt-2 dark:border-gray-700",
                    span {
                        class: "text-[10px] font-medium uppercase tracking-wider text-gray-400",
                        "Rating"
                    }
                    {
                        let rating_options: Vec<(Option<i32>, &str)> = vec![
                            (None, "Any Rating"),
                            (Some(3), "3+"),
                            (Some(4), "4+"),
                            (Some(5), "5"),
                        ];
                        rsx! {
                            for (opt_value, label) in rating_options.into_iter() {
                                {
                                    let is_active = props.min_rating == opt_value;
                                    let active_class = if is_active {
                                        "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                                    } else {
                                        "bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                                    };
                                    rsx! {
                                        button {
                                            key: "{label}",
                                            r#type: "button",
                                            onclick: move |_| props.on_min_rating_change.call(opt_value),
                                            class: "rounded-full px-3 py-1.5 text-xs font-medium active:scale-95 transition-all duration-150 {active_class}",
                                            if opt_value.is_some() {
                                                span {
                                                    class: "text-[10px] opacity-60",
                                                    "\u{2605}"
                                                }
                                            }
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Shelf row
            if has_shelves {
                div {
                    class: "flex flex-wrap items-center gap-1.5 border-t border-gray-200 pt-2 dark:border-gray-700",
                    span {
                        class: "text-[10px] font-medium uppercase tracking-wider text-gray-400",
                        "Shelves"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| props.on_shelf_change.call(None),
                        class: if props.active_shelf.is_none() {
                            "inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium active:scale-95 transition-all duration-150 bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                        } else {
                            "inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium active:scale-95 transition-all duration-150 bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                        },
                        "All Shelves"
                    }
                    for shelf in props.shelves.iter() {
                        {
                            let is_active = props.active_shelf == Some(shelf.id);
                            let count = props.shelf_book_counts.get(&shelf.id).copied().unwrap_or(0);
                            let active_class = if is_active {
                                "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                            } else {
                                "bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                            };
                            let count_class = if is_active {
                                "bg-white/40 dark:bg-black/20"
                            } else {
                                "bg-gray-200/80 dark:bg-gray-700"
                            };
                            let sid = shelf.id;
                            rsx! {
                                button {
                                    key: "{shelf.id}",
                                    r#type: "button",
                                    onclick: move |_| props.on_shelf_change.call(Some(sid)),
                                    class: "inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium active:scale-95 transition-all duration-150 {active_class}",
                                    "{shelf.name}"
                                    span {
                                        class: "inline-flex h-4.5 min-w-[1.125rem] items-center justify-center rounded-full px-1 text-[10px] font-semibold leading-none {count_class}",
                                        "{count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Active filter summary
            if has_active_filters {
                div {
                    class: "flex flex-wrap items-center gap-1.5 border-t border-gray-200 pt-2 dark:border-gray-700",
                    span {
                        class: "text-[10px] font-medium uppercase tracking-wider text-gray-400",
                        "Filters"
                    }
                    if props.active_status != "all" {
                        {
                            let label = FILTER_STATUSES.iter().find(|(v, _)| *v == props.active_status).map(|(_, l)| *l).unwrap_or("");
                            rsx! {
                                FilterTag {
                                    label: label.to_string(),
                                    on_dismiss: move |_| props.on_status_change.call("all".to_string()),
                                }
                            }
                        }
                    }
                    if let Some(min) = props.min_rating {
                        FilterTag {
                            label: format!("{}+ \u{2605}", min),
                            on_dismiss: move |_| props.on_min_rating_change.call(None),
                        }
                    }
                    if let Some(shelf_id) = props.active_shelf {
                        {
                            let name = props.shelves.iter().find(|s| s.id == shelf_id).map(|s| s.name.clone()).unwrap_or_default();
                            rsx! {
                                FilterTag {
                                    label: name,
                                    on_dismiss: move |_| props.on_shelf_change.call(None),
                                }
                            }
                        }
                    }
                    if !props.search_query.is_empty() {
                        FilterTag {
                            label: format!("search: {}", props.search_query),
                            on_dismiss: move |_| props.on_search_change.call(String::new()),
                        }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| props.on_clear_all.call(()),
                        class: "text-xs font-medium text-amber-700 hover:text-amber-900 dark:text-amber-400 dark:hover:text-amber-200",
                        "Clear all"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FilterTagProps {
    label: String,
    on_dismiss: EventHandler<()>,
}

#[component]
fn FilterTag(props: FilterTagProps) -> Element {
    rsx! {
        span {
            class: "inline-flex items-center gap-1 rounded-full bg-amber-50 px-2.5 py-1 text-xs font-medium text-amber-800 dark:bg-amber-900/30 dark:text-amber-300",
            "{props.label}"
            button {
                r#type: "button",
                onclick: move |_| props.on_dismiss.call(()),
                class: "ml-0.5 text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200",
                "\u{00d7}"
            }
        }
    }
}
