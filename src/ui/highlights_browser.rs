use std::time::Duration;

use dioxus::prelude::*;

use crate::data::commands::list_all_highlights_db;
use crate::data::models::HighlightSearchResult;
use crate::services::share_card::{open_share_sheet, ShareCardData};
use crate::DatabaseHandle;

fn truncate_text(text: &str, max_len: usize) -> String {
    let trimmed = text.trim();
    match trimmed.char_indices().nth(max_len) {
        Some((byte_idx, _)) => format!("{}...", &trimmed[..byte_idx]),
        None => trimmed.to_string(),
    }
}

fn clip_type_badge_class(clip_type: &str) -> &'static str {
    match clip_type.to_lowercase().as_str() {
        "note" => "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300",
        "bookmark" => "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300",
        _ => "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300",
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct HighlightsBrowserProps {
    on_select_book: EventHandler<i64>,
}

#[component]
pub fn HighlightsBrowser(props: HighlightsBrowserProps) -> Element {
    let db = use_context::<DatabaseHandle>();
    let mut highlights: Signal<Vec<HighlightSearchResult>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut search_query = use_signal(String::new);
    let mut active_clip_type: Signal<Option<String>> = use_signal(|| None);
    let mut copied_id: Signal<Option<i64>> = use_signal(|| None);

    {
        let db = db.clone();
        use_effect(move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                match list_all_highlights_db(&conn) {
                    Ok(h) => highlights.set(h),
                    Err(e) => tracing::error!("Failed to load highlights: {e}"),
                }
                loading.set(false);
            });
        });
    }

    let filtered = use_memo(move || {
        let all = highlights.read();
        let query = search_query.read().to_lowercase();
        let clip_filter = active_clip_type.read().clone();

        all.iter()
            .filter(|h| {
                if let Some(ref ct) = clip_filter {
                    if h.clip_type.to_lowercase() != *ct {
                        return false;
                    }
                }
                if !query.is_empty() {
                    let searchable = format!(
                        "{} {} {}",
                        h.text,
                        h.book_title,
                        h.book_author.as_deref().unwrap_or("")
                    )
                    .to_lowercase();
                    if !searchable.contains(&query) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    if *loading.read() && highlights.read().is_empty() {
        return rsx! {
            div {
                class: "flex items-center justify-center py-20",
                p { class: "text-sm text-gray-400 dark:text-gray-500", "Loading highlights..." }
            }
        };
    }

    if highlights.read().is_empty() {
        return rsx! {
            div {
                class: "flex flex-col items-center justify-center py-20",
                p { class: "text-sm text-gray-400 dark:text-gray-500", "No highlights yet \u{2014} sync your Kindle to import highlights." }
            }
        };
    }

    let filter_buttons: Vec<(&str, Option<String>)> = vec![
        ("All", None),
        ("Highlights", Some("highlight".to_string())),
        ("Notes", Some("note".to_string())),
        ("Bookmarks", Some("bookmark".to_string())),
    ];

    let count = filtered.read().len();
    let count_label = if count == 1 {
        "1 highlight".to_string()
    } else {
        format!("{count} highlights")
    };

    rsx! {
        div {
            class: "mx-auto max-w-2xl px-4 py-6 flex flex-col h-full",

            div {
                class: "mb-4",
                input {
                    r#type: "text",
                    placeholder: "Search highlights...",
                    class: "w-full rounded-lg border border-gray-200 bg-white px-3 py-2 text-sm
                        text-gray-900 placeholder-gray-400 focus:border-amber-400 focus:outline-none
                        focus:ring-1 focus:ring-amber-400 dark:border-gray-700 dark:bg-gray-900
                        dark:text-gray-100 dark:placeholder-gray-500",
                    oninput: move |e| search_query.set(e.value()),
                }
            }

            div {
                class: "mb-4 flex gap-2",
                for (label, clip_value) in filter_buttons.iter() {
                    {
                        let is_active = *active_clip_type.read() == *clip_value;
                        let cv = clip_value.clone();
                        let btn_class = if is_active {
                            "rounded-md px-3 py-1 text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                        } else {
                            "rounded-md px-3 py-1 text-xs font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                        };
                        rsx! {
                            button {
                                r#type: "button",
                                class: "{btn_class}",
                                onclick: move |_| active_clip_type.set(cv.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }

            p {
                class: "mb-3 text-xs text-gray-500 dark:text-gray-400",
                "{count_label}"
            }

            if count == 0 {
                div {
                    class: "flex items-center justify-center py-12",
                    p { class: "text-sm text-gray-400 dark:text-gray-500", "No highlights match your search." }
                }
            } else {
                div {
                    class: "overflow-y-auto space-y-3",
                    for highlight in filtered.read().iter() {
                        {
                            let hid = highlight.id;
                            let book_id = highlight.book_id;
                            let display_text = truncate_text(&highlight.text, 200);
                            let full_text = highlight.text.clone();
                            let title = highlight.book_title.clone();
                            let author = highlight.book_author.clone();
                            let clip_type = highlight.clip_type.clone();
                            let badge_class = clip_type_badge_class(&clip_type);
                            let page = highlight.page;
                            let loc_start = highlight.location_start;
                            let loc_end = highlight.location_end;
                            let is_copied = *copied_id.read() == Some(hid);
                            let share_text = full_text.clone();
                            let share_title = title.clone();
                            let share_author = author.clone().unwrap_or_default();
                            let book_rating = highlight.book_rating;

                            rsx! {
                                div {
                                    key: "{hid}",
                                    class: "relative flex w-full flex-col gap-2 rounded-lg border border-gray-200
                                        bg-white p-4 text-left transition hover:border-amber-300
                                        hover:shadow-sm dark:border-gray-700 dark:bg-gray-900
                                        dark:hover:border-amber-600 cursor-pointer",
                                    onclick: move |_| props.on_select_book.call(book_id),

                                    div {
                                        class: "flex items-start justify-between gap-2",
                                        p {
                                            class: "text-sm text-gray-800 dark:text-gray-200 leading-relaxed",
                                            "{display_text}"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "flex-shrink-0 rounded px-2 py-1 text-xs text-gray-400
                                                hover:text-gray-600 hover:bg-gray-100 dark:hover:text-gray-300
                                                dark:hover:bg-gray-800 transition",
                                            onclick: move |e: Event<MouseData>| {
                                                e.stop_propagation();
                                                let text = full_text.clone();
                                                match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
                                                    Ok(()) => {
                                                        copied_id.set(Some(hid));
                                                        spawn(async move {
                                                            tokio::time::sleep(Duration::from_millis(1500)).await;
                                                            if *copied_id.read() == Some(hid) {
                                                                copied_id.set(None);
                                                            }
                                                        });
                                                    }
                                                    Err(err) => {
                                                        tracing::warn!("Clipboard error: {err}");
                                                    }
                                                }
                                            },
                                            if is_copied { "Copied!" } else { "Copy" }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "flex-shrink-0 rounded px-2 py-1 text-xs text-gray-400
                                                hover:text-gray-600 hover:bg-gray-100 dark:hover:text-gray-300
                                                dark:hover:bg-gray-800 transition",
                                            onclick: move |e: Event<MouseData>| {
                                                e.stop_propagation();
                                                let quote = share_text.clone();
                                                let book_title = share_title.clone();
                                                let author = share_author.clone();
                                                spawn(async move {
                                                    let data = ShareCardData::Highlight {
                                                        quote,
                                                        book_title,
                                                        author,
                                                        rating: book_rating,
                                                    };
                                                    let name = format!("highlight-{hid}");
                                                    let result = tokio::task::spawn_blocking(move || {
                                                        open_share_sheet(&data, &name)
                                                    })
                                                    .await;
                                                    match result {
                                                        Ok(Ok(())) => {}
                                                        Ok(Err(err)) => tracing::warn!("Share card error: {err}"),
                                                        Err(err) => tracing::warn!("Share card task error: {err}"),
                                                    }
                                                });
                                            },
                                            "Share"
                                        }
                                    }

                                    div {
                                        class: "flex items-center justify-between gap-2",
                                        div {
                                            class: "min-w-0",
                                            p {
                                                class: "truncate text-xs font-medium text-gray-600 dark:text-gray-300",
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
                                            class: "flex-shrink-0 rounded px-2 py-0.5 text-xs font-medium {badge_class}",
                                            "{clip_type}"
                                        }
                                    }

                                    {
                                        let mut location_parts: Vec<String> = Vec::new();
                                        if let Some(p) = page {
                                            location_parts.push(format!("Page {p}"));
                                        }
                                        if let Some(start) = loc_start {
                                            if let Some(end) = loc_end {
                                                location_parts.push(format!("Loc {start}-{end}"));
                                            } else {
                                                location_parts.push(format!("Loc {start}"));
                                            }
                                        }
                                        if !location_parts.is_empty() {
                                            rsx! {
                                                p {
                                                    class: "text-xs text-gray-400 dark:text-gray-500",
                                                    "{location_parts.join(\" \u{00b7} \")}"
                                                }
                                            }
                                        } else {
                                            rsx! {}
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

#[cfg(test)]
#[path = "highlights_browser_tests.rs"]
mod tests;
