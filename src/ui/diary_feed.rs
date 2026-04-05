use dioxus::prelude::*;

use crate::data::commands::list_diary_entries_db;
use crate::data::models::DiaryEntry;
use crate::DatabaseHandle;

use super::rating_stars::RatingStars;

fn format_date(date_str: &str) -> String {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return date_str.to_string();
    }
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let month: usize = parts[1].parse().unwrap_or(1);
    let day: u32 = parts[2].parse().unwrap_or(1);
    let year = parts[0];
    let month_name = month_names.get(month.wrapping_sub(1)).unwrap_or(&"?");
    format!("{month_name} {day}, {year}")
}

fn month_key(date_str: &str) -> String {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() < 2 {
        return date_str.to_string();
    }
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let month: usize = parts[1].parse().unwrap_or(1);
    let year = parts[0];
    let month_name = month_names.get(month.wrapping_sub(1)).unwrap_or(&"?");
    format!("{month_name} {year}")
}

fn extract_plain_text(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.len() > 150 {
        format!("{}...", &trimmed[..150])
    } else {
        trimmed.to_string()
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DiaryFeedProps {
    on_select_entry: EventHandler<DiaryEntry>,
}

#[component]
pub fn DiaryFeed(props: DiaryFeedProps) -> Element {
    let db = use_context::<DatabaseHandle>();
    let mut entries: Signal<Vec<DiaryEntry>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);

    {
        let db = db.clone();
        use_effect(move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                if let Ok(e) = list_diary_entries_db(&conn) {
                    entries.set(e);
                }
                loading.set(false);
            });
        });
    }

    if *loading.read() && entries.read().is_empty() {
        return rsx! {
            div {
                class: "flex items-center justify-center py-20",
                p { class: "text-sm text-gray-400 dark:text-gray-500", "Loading diary..." }
            }
        };
    }

    if entries.read().is_empty() {
        return rsx! {
            div {
                class: "flex flex-col items-center justify-center py-20",
                p { class: "text-sm text-gray-400 dark:text-gray-500", "No diary entries yet" }
                p {
                    class: "mt-1 text-xs text-gray-400 dark:text-gray-600",
                    "Add entries from a book's detail page"
                }
            }
        };
    }

    let entries_read = entries.read();
    let mut grouped: Vec<(String, Vec<&DiaryEntry>)> = Vec::new();
    let mut current_month = String::new();
    for entry in entries_read.iter() {
        let mk = month_key(&entry.entry_date);
        if mk != current_month {
            current_month = mk.clone();
            grouped.push((mk, Vec::new()));
        }
        if let Some(last) = grouped.last_mut() {
            last.1.push(entry);
        }
    }

    rsx! {
        div {
            class: "mx-auto max-w-2xl px-4 py-6",
            for (month, group_entries) in grouped.iter() {
                div {
                    key: "{month}",
                    class: "mb-8",
                    h2 {
                        class: "mb-3 text-sm font-semibold text-gray-500 dark:text-gray-400",
                        "{month}"
                    }
                    div {
                        class: "space-y-3",
                        for entry in group_entries.iter() {
                            {
                                let title = entry.book_title.clone();
                                let author = entry.book_author.clone();
                                let cover_url = entry.book_cover_url.clone();
                                let date = format_date(&entry.entry_date);
                                let rating = entry.rating;
                                let body_preview = entry.body.as_deref().map(extract_plain_text);
                                let first_char = title.chars().next().unwrap_or('?').to_uppercase().to_string();
                                let entry_clone = (*entry).clone();
                                let eid = entry.id;

                                rsx! {
                                    button {
                                        key: "{eid}",
                                        r#type: "button",
                                        onclick: move |_| props.on_select_entry.call(entry_clone.clone()),
                                        class: "flex w-full gap-3 rounded-lg border border-gray-200 bg-white
                                            p-3 text-left transition hover:border-amber-300 hover:shadow-sm
                                            dark:border-gray-700 dark:bg-gray-900 dark:hover:border-amber-600",

                                        div {
                                            class: "h-16 w-11 flex-shrink-0 overflow-hidden rounded bg-gray-100 dark:bg-gray-700",
                                            if let Some(ref url) = cover_url {
                                                img {
                                                    src: "{url}",
                                                    alt: "{title}",
                                                    class: "h-full w-full object-cover",
                                                }
                                            } else {
                                                div {
                                                    class: "flex h-full w-full items-center justify-center
                                                        bg-gradient-to-br from-amber-100 to-orange-200
                                                        dark:from-amber-900/40 dark:to-orange-900/40",
                                                    span {
                                                        class: "text-sm font-bold text-amber-700/60 dark:text-amber-400/60",
                                                        "{first_char}"
                                                    }
                                                }
                                            }
                                        }

                                        div {
                                            class: "min-w-0 flex-1",
                                            div {
                                                class: "flex items-start justify-between gap-2",
                                                div {
                                                    class: "min-w-0",
                                                    p {
                                                        class: "truncate text-sm font-medium text-gray-900 dark:text-gray-100",
                                                        "{title}"
                                                    }
                                                    if let Some(ref a) = author {
                                                        p {
                                                            class: "truncate text-xs text-gray-500 dark:text-gray-400",
                                                            "{a}"
                                                        }
                                                    }
                                                }
                                                span {
                                                    class: "flex-shrink-0 text-xs text-gray-400 dark:text-gray-500",
                                                    "{date}"
                                                }
                                            }

                                            if let Some(r) = rating {
                                                div {
                                                    class: "mt-1",
                                                    RatingStars { rating: r, on_rate: move |_: i32| {}, small: true }
                                                }
                                            }

                                            if let Some(ref preview) = body_preview {
                                                p {
                                                    class: "mt-1 text-xs leading-relaxed text-gray-600 dark:text-gray-400 line-clamp-2",
                                                    "{preview}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
