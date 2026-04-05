use dioxus::prelude::*;

use crate::data::commands::{import_goodreads_db, ImportStats};
use crate::services::goodreads::{
    collect_unique_shelves, parse_goodreads_csv, GoodreadsParseResult,
};
use crate::DatabaseHandle;

#[derive(Clone, PartialEq)]
enum Phase {
    SelectFile,
    Preview,
    Importing,
    Done,
    Error,
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct GoodreadsImportProps {
    on_close: EventHandler<()>,
    on_import_complete: EventHandler<()>,
}

#[component]
pub fn GoodreadsImport(props: GoodreadsImportProps) -> Element {
    let db = use_context::<DatabaseHandle>();

    let mut phase = use_signal(|| Phase::SelectFile);
    let mut parse_result: Signal<Option<GoodreadsParseResult>> = use_signal(|| None);
    let mut unique_shelves: Signal<Vec<String>> = use_signal(Vec::new);
    let mut stats: Signal<Option<ImportStats>> = use_signal(|| None);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let handle_pick_file = move |_| {
        spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_title("Select Goodreads CSV Export")
                .add_filter("CSV files", &["csv"])
                .pick_file()
                .await;

            if let Some(file) = file {
                let path = file.path().to_path_buf();
                match std::fs::read_to_string(&path) {
                    Ok(content) => match parse_goodreads_csv(content.as_bytes()) {
                        Ok(result) => {
                            let shelves = collect_unique_shelves(&result.books);
                            unique_shelves.set(shelves);
                            parse_result.set(Some(result));
                            phase.set(Phase::Preview);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to parse CSV: {e}")));
                            phase.set(Phase::Error);
                        }
                    },
                    Err(e) => {
                        error.set(Some(format!("Failed to read file: {e}")));
                        phase.set(Phase::Error);
                    }
                }
            }
        });
    };

    let handle_import = move |_| {
        let db = db.clone();
        phase.set(Phase::Importing);
        spawn(async move {
            let parsed = parse_result.read().clone();
            if let Some(parsed) = parsed {
                let mut conn = db.conn.lock().unwrap();
                match import_goodreads_db(&mut conn, &parsed.books) {
                    Ok(result) => {
                        stats.set(Some(result));
                        phase.set(Phase::Done);
                        props.on_import_complete.call(());
                    }
                    Err(e) => {
                        error.set(Some(format!("Import failed: {e}")));
                        phase.set(Phase::Error);
                    }
                }
            }
        });
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "w-full max-w-lg rounded-2xl bg-white p-6 shadow-2xl dark:bg-gray-900",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "mb-4 flex items-center justify-between",
                    h2 {
                        class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                        "Import from Goodreads"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| props.on_close.call(()),
                        class: "rounded-full p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600
                            dark:hover:bg-gray-800 dark:hover:text-gray-300",
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

                match *phase.read() {
                    Phase::SelectFile => rsx! {
                        div {
                            class: "flex flex-col items-center gap-4 py-8",
                            div {
                                class: "text-center text-sm text-gray-500 dark:text-gray-400",
                                p { "Export your Goodreads library as CSV from:" }
                                p {
                                    class: "mt-1 font-mono text-xs text-gray-400",
                                    "goodreads.com/review/import"
                                }
                            }
                            button {
                                r#type: "button",
                                onclick: handle_pick_file,
                                class: "rounded-lg bg-amber-600 px-5 py-2.5 text-sm font-medium
                                    text-white shadow-sm transition hover:bg-amber-700 active:scale-95",
                                "Select CSV File"
                            }
                        }
                    },
                    Phase::Preview => rsx! {
                        {render_preview(&parse_result.read(), &unique_shelves.read())}
                        div {
                            class: "mt-4 flex justify-end gap-3",
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    parse_result.set(None);
                                    phase.set(Phase::SelectFile);
                                },
                                class: "rounded-lg px-4 py-2 text-sm font-medium text-gray-600
                                    transition hover:bg-gray-100 dark:text-gray-400
                                    dark:hover:bg-gray-800",
                                "Back"
                            }
                            button {
                                r#type: "button",
                                onclick: handle_import,
                                class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                    text-white shadow-sm transition hover:bg-amber-700 active:scale-95",
                                "Import"
                            }
                        }
                    },
                    Phase::Importing => rsx! {
                        div {
                            class: "flex flex-col items-center gap-3 py-8",
                            div {
                                class: "h-8 w-8 animate-spin rounded-full border-2 border-amber-200 border-t-amber-600",
                            }
                            p {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "Importing books..."
                            }
                        }
                    },
                    Phase::Done => rsx! {
                        {render_results(&stats.read())}
                        div {
                            class: "mt-4 flex justify-end",
                            button {
                                r#type: "button",
                                onclick: move |_| props.on_close.call(()),
                                class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                    text-white shadow-sm transition hover:bg-amber-700 active:scale-95",
                                "Done"
                            }
                        }
                    },
                    Phase::Error => rsx! {
                        div {
                            class: "py-6",
                            div {
                                class: "rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400",
                                {error.read().as_deref().unwrap_or("Unknown error")}
                            }
                            div {
                                class: "mt-4 flex justify-end gap-3",
                                button {
                                    r#type: "button",
                                    onclick: move |_| {
                                        error.set(None);
                                        phase.set(Phase::SelectFile);
                                    },
                                    class: "rounded-lg px-4 py-2 text-sm font-medium text-gray-600
                                        transition hover:bg-gray-100 dark:text-gray-400
                                        dark:hover:bg-gray-800",
                                    "Try Again"
                                }
                                button {
                                    r#type: "button",
                                    onclick: move |_| props.on_close.call(()),
                                    class: "rounded-lg px-4 py-2 text-sm font-medium text-gray-600
                                        transition hover:bg-gray-100 dark:text-gray-400
                                        dark:hover:bg-gray-800",
                                    "Close"
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

fn render_preview(
    parse_result: &Option<GoodreadsParseResult>,
    unique_shelves: &[String],
) -> Element {
    let result = match parse_result {
        Some(r) => r,
        None => return rsx! {},
    };

    let book_count = result.books.len();
    let with_rating = result.books.iter().filter(|b| b.rating.is_some()).count();
    let with_review = result.books.iter().filter(|b| b.review.is_some()).count();
    let finished = result
        .books
        .iter()
        .filter(|b| b.status == "finished")
        .count();
    let reading = result
        .books
        .iter()
        .filter(|b| b.status == "reading")
        .count();
    let want = result
        .books
        .iter()
        .filter(|b| b.status == "want_to_read")
        .count();
    let shelf_count = unique_shelves.len();

    rsx! {
        div {
            class: "space-y-3",
            // Summary card
            div {
                class: "rounded-lg bg-amber-50 p-4 dark:bg-amber-900/20",
                p {
                    class: "text-sm font-medium text-amber-800 dark:text-amber-300",
                    "Found {book_count} book{plural(book_count)} to import"
                }
                if result.skipped_rows > 0 {
                    p {
                        class: "mt-1 text-xs text-amber-600 dark:text-amber-400",
                        "{result.skipped_rows} row{plural(result.skipped_rows)} could not be parsed"
                    }
                }
            }

            // Breakdown
            div {
                class: "grid grid-cols-2 gap-2 text-sm",
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "Finished" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{finished}" }
                }
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "Reading" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{reading}" }
                }
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "Want to Read" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{want}" }
                }
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "With Ratings" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{with_rating}" }
                }
            }

            if with_review > 0 {
                p {
                    class: "text-xs text-gray-500 dark:text-gray-400",
                    "{with_review} review{plural(with_review)} will be imported as diary entries"
                }
            }

            if shelf_count > 0 {
                div {
                    class: "text-xs text-gray-500 dark:text-gray-400",
                    span { "{shelf_count} shelf/shelves: " }
                    span {
                        class: "text-gray-600 dark:text-gray-300",
                        {unique_shelves.join(", ")}
                    }
                }
            }
        }
    }
}

fn render_results(stats: &Option<ImportStats>) -> Element {
    let stats = match stats {
        Some(s) => s,
        None => return rsx! {},
    };

    rsx! {
        div {
            class: "space-y-3 py-4",
            div {
                class: "rounded-lg bg-green-50 p-4 dark:bg-green-900/20",
                p {
                    class: "text-sm font-medium text-green-800 dark:text-green-300",
                    "Import complete!"
                }
            }
            div {
                class: "grid grid-cols-2 gap-2 text-sm",
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "Books Imported" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{stats.books_imported}" }
                }
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "Books Skipped" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{stats.books_skipped}" }
                }
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "Diary Entries" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{stats.entries_created}" }
                }
                div {
                    class: "rounded-lg bg-gray-50 p-3 dark:bg-gray-800",
                    p { class: "text-xs text-gray-400 dark:text-gray-500", "Shelves Created" }
                    p { class: "font-medium text-gray-900 dark:text-gray-100", "{stats.shelves_created}" }
                }
            }
        }
    }
}
