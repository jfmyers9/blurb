use dioxus::prelude::*;

use crate::services::letterboxd::{
    merge_entries, parse_diary_csv, parse_ratings_csv, LetterboxdEntry,
};

#[derive(Props, Clone, PartialEq)]
pub struct LetterboxdImportProps {
    on_close: EventHandler<()>,
}

#[component]
pub fn LetterboxdImport(props: LetterboxdImportProps) -> Element {
    let mut diary_entries: Signal<Vec<LetterboxdEntry>> = use_signal(Vec::new);
    let mut ratings_entries: Signal<Vec<LetterboxdEntry>> = use_signal(Vec::new);
    let mut merged: Signal<Vec<LetterboxdEntry>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut diary_loaded = use_signal(|| false);
    let mut ratings_loaded = use_signal(|| false);

    let mut recompute_merge = move || {
        let m = merge_entries(diary_entries.read().clone(), ratings_entries.read().clone());
        merged.set(m);
    };

    let handle_diary_pick = move |_| {
        spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_title("Select Letterboxd diary.csv")
                .add_filter("CSV", &["csv"])
                .pick_file()
                .await;
            if let Some(f) = file {
                let bytes = f.read().await;
                match String::from_utf8(bytes) {
                    Ok(content) => match parse_diary_csv(&content) {
                        Ok(entries) => {
                            diary_entries.set(entries);
                            diary_loaded.set(true);
                            error.set(None);
                            recompute_merge();
                        }
                        Err(e) => error.set(Some(format!("Failed to parse diary.csv: {e}"))),
                    },
                    Err(e) => error.set(Some(format!("Invalid UTF-8: {e}"))),
                }
            }
        });
    };

    let handle_ratings_pick = move |_| {
        spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_title("Select Letterboxd ratings.csv")
                .add_filter("CSV", &["csv"])
                .pick_file()
                .await;
            if let Some(f) = file {
                let bytes = f.read().await;
                match String::from_utf8(bytes) {
                    Ok(content) => match parse_ratings_csv(&content) {
                        Ok(entries) => {
                            ratings_entries.set(entries);
                            ratings_loaded.set(true);
                            error.set(None);
                            recompute_merge();
                        }
                        Err(e) => error.set(Some(format!("Failed to parse ratings.csv: {e}"))),
                    },
                    Err(e) => error.set(Some(format!("Invalid UTF-8: {e}"))),
                }
            }
        });
    };

    let preview = merged.read();
    let has_entries = !preview.is_empty();

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4",
            div {
                class: "flex max-h-[80vh] w-full max-w-2xl flex-col rounded-xl
                    border border-gray-700 bg-gray-900 shadow-2xl",

                // Header
                div {
                    class: "flex items-center justify-between border-b border-gray-700 px-5 py-4",
                    h2 { class: "text-lg font-semibold text-gray-100", "Letterboxd Import" }
                    button {
                        r#type: "button",
                        onclick: move |_| props.on_close.call(()),
                        class: "text-gray-400 hover:text-gray-200",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M6 18L18 6M6 6l12 12",
                            }
                        }
                    }
                }

                // Body
                div {
                    class: "flex-1 overflow-y-auto px-5 py-4",

                    if let Some(ref err) = *error.read() {
                        div {
                            class: "mb-4 rounded-lg bg-red-900/50 px-4 py-2 text-sm text-red-300",
                            "{err}"
                        }
                    }

                    // File pickers
                    div {
                        class: "mb-4 space-y-3",
                        div {
                            class: "flex items-center gap-3",
                            button {
                                r#type: "button",
                                onclick: handle_diary_pick,
                                class: "rounded-lg bg-orange-600 px-4 py-2 text-sm font-medium text-white
                                    hover:bg-orange-500 transition-colors",
                                "Load diary.csv"
                            }
                            if *diary_loaded.read() {
                                span {
                                    class: "text-sm text-green-400",
                                    "{diary_entries.read().len()} diary entries loaded"
                                }
                            }
                        }
                        div {
                            class: "flex items-center gap-3",
                            button {
                                r#type: "button",
                                onclick: handle_ratings_pick,
                                class: "rounded-lg bg-orange-600/80 px-4 py-2 text-sm font-medium text-white
                                    hover:bg-orange-500 transition-colors",
                                "Load ratings.csv (optional)"
                            }
                            if *ratings_loaded.read() {
                                span {
                                    class: "text-sm text-green-400",
                                    "{ratings_entries.read().len()} ratings loaded"
                                }
                            }
                        }
                    }

                    // Summary
                    if has_entries {
                        div {
                            class: "mb-4 rounded-lg bg-gray-800 px-4 py-3",
                            p {
                                class: "text-sm text-gray-300",
                                "{preview.len()} unique movies found"
                            }
                            p {
                                class: "text-xs text-gray-500 mt-1",
                                {
                                    let rated = preview.iter().filter(|e| e.rating.is_some()).count();
                                    let watched = preview.iter().filter(|e| e.watched_date.is_some()).count();
                                    let rewatches = preview.iter().filter(|e| e.rewatch).count();
                                    let tagged = preview.iter().filter(|e| !e.tags.is_empty()).count();
                                    format!("{rated} rated, {watched} with watch dates, {rewatches} rewatches, {tagged} tagged")
                                }
                            }
                        }

                        // Preview table
                        div {
                            class: "rounded-lg border border-gray-700 overflow-hidden",
                            table {
                                class: "w-full text-sm",
                                thead {
                                    tr {
                                        class: "bg-gray-800 text-left text-xs text-gray-400",
                                        th { class: "px-3 py-2", "Title" }
                                        th { class: "px-3 py-2 w-16", "Year" }
                                        th { class: "px-3 py-2 w-12", "Stars" }
                                        th { class: "px-3 py-2 w-28", "Watched" }
                                    }
                                }
                                tbody {
                                    for (i, entry) in preview.iter().take(50).enumerate() {
                                        tr {
                                            key: "{i}",
                                            class: if i % 2 == 0 { "bg-gray-900" } else { "bg-gray-800/50" },
                                            td {
                                                class: "px-3 py-1.5 text-gray-200 truncate max-w-[200px]",
                                                "{entry.title}"
                                            }
                                            td {
                                                class: "px-3 py-1.5 text-gray-400",
                                                {entry.year.map(|y| format!("{y}")).unwrap_or_default()}
                                            }
                                            td {
                                                class: "px-3 py-1.5 text-gray-400",
                                                title: entry.letterboxd_uri.clone().unwrap_or_default(),
                                                {entry.rating_int.map(|r| format!("{r}/5")).unwrap_or_default()}
                                            }
                                            td {
                                                class: "px-3 py-1.5 text-gray-500",
                                                {entry.watched_date.clone().unwrap_or_default()}
                                            }
                                        }
                                    }
                                }
                            }
                            if preview.len() > 50 {
                                div {
                                    class: "px-3 py-2 text-xs text-gray-500 bg-gray-800 border-t border-gray-700",
                                    "Showing 50 of {preview.len()} entries"
                                }
                            }
                        }
                    }
                }

                // Footer
                div {
                    class: "flex items-center justify-end gap-3 border-t border-gray-700 px-5 py-3",
                    button {
                        r#type: "button",
                        onclick: move |_| props.on_close.call(()),
                        class: "rounded-lg bg-gray-700 px-4 py-2 text-sm text-gray-300
                            hover:bg-gray-600 transition-colors",
                        "Close"
                    }
                    button {
                        r#type: "button",
                        disabled: true,
                        class: "rounded-lg bg-gray-600 px-4 py-2 text-sm text-gray-400
                            cursor-not-allowed opacity-60",
                        "Movie support coming in v0.3.0"
                    }
                }
            }
        }
    }
}
