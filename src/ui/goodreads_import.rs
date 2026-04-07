use dioxus::prelude::*;

use crate::data::commands::{count_goodreads_duplicates_db, import_goodreads_books_db};
use crate::services::enrichment::{
    run_enrichment, EnrichmentState, EnrichmentStatus, ImportSource,
};
use crate::services::goodreads::{parse_goodreads_csv, GoodreadsBook};
use crate::DatabaseHandle;

#[derive(Clone, PartialEq)]
enum Phase {
    FileSelect,
    Parsing,
    Confirm,
    Importing,
    Done,
}

#[derive(Props, Clone, PartialEq)]
pub struct GoodreadsImportProps {
    on_close: EventHandler<()>,
    on_import_complete: EventHandler<()>,
}

#[component]
pub fn GoodreadsImport(props: GoodreadsImportProps) -> Element {
    let db = use_context::<DatabaseHandle>();
    let enrichment_state = use_context::<EnrichmentState>();

    let mut phase = use_signal(|| Phase::FileSelect);
    let mut parsed_books: Signal<Vec<GoodreadsBook>> = use_signal(Vec::new);
    let mut duplicate_count = use_signal(|| 0usize);
    let mut imported_count = use_signal(|| 0usize);
    let mut skipped_count = use_signal(|| 0usize);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let mut include_finished = use_signal(|| true);
    let mut include_reading = use_signal(|| true);
    let mut include_want_to_read = use_signal(|| true);
    let mut import_ratings = use_signal(|| true);
    let mut import_read_dates = use_signal(|| true);
    let mut enrich = use_signal(|| true);
    let mut enrichment_warning = use_signal(|| false);

    let handle_pick_file = {
        let db = db.clone();
        move |_| {
            let db = db.clone();
            phase.set(Phase::Parsing);
            error.set(None);
            spawn(async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("CSV", &["csv"])
                    .set_title("Select Goodreads Export CSV")
                    .pick_file()
                    .await;

                let Some(file) = file else {
                    phase.set(Phase::FileSelect);
                    return;
                };

                let path = file.path().to_path_buf();
                match parse_goodreads_csv(&path) {
                    Ok(books) => {
                        let dupes = {
                            let conn = db.conn.lock().unwrap();
                            count_goodreads_duplicates_db(&conn, &books).unwrap_or(0)
                        };
                        duplicate_count.set(dupes);
                        parsed_books.set(books);
                        phase.set(Phase::Confirm);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to parse CSV: {e}")));
                        phase.set(Phase::FileSelect);
                    }
                }
            });
        }
    };

    let handle_import = {
        let db = db.clone();
        let on_import_complete = props.on_import_complete;
        move |_| {
            let db = db.clone();
            let should_enrich = *enrich.read();
            let mut books: Vec<GoodreadsBook> = parsed_books
                .read()
                .iter()
                .filter(|b| match b.status.as_str() {
                    "finished" => *include_finished.read(),
                    "reading" => *include_reading.read(),
                    "want_to_read" => *include_want_to_read.read(),
                    _ => true,
                })
                .cloned()
                .collect();
            if !*import_ratings.read() {
                for book in &mut books {
                    book.rating = None;
                }
            }
            if !*import_read_dates.read() {
                for book in &mut books {
                    book.date_read = None;
                }
            }
            phase.set(Phase::Importing);
            error.set(None);
            spawn(async move {
                let result = {
                    let mut conn = db.conn.lock().unwrap();
                    import_goodreads_books_db(&mut conn, &books)
                };
                match result {
                    Ok(res) => {
                        imported_count.set(res.imported_count);
                        skipped_count.set(res.skipped_count);
                        on_import_complete.call(());
                        if should_enrich && !res.new_book_ids.is_empty() {
                            let is_running = matches!(
                                *enrichment_state.status.read(),
                                EnrichmentStatus::Running { .. }
                            );
                            if is_running {
                                enrichment_warning.set(true);
                            } else {
                                spawn(run_enrichment(
                                    db.clone(),
                                    enrichment_state,
                                    res.new_book_ids,
                                    ImportSource::Goodreads,
                                ));
                            }
                        }
                        phase.set(Phase::Done);
                    }
                    Err(e) => {
                        error.set(Some(format!("Import failed: {e}")));
                        phase.set(Phase::Confirm);
                    }
                }
            });
        }
    };

    let total = parsed_books.read().len();
    let filtered_count = parsed_books
        .read()
        .iter()
        .filter(|b| match b.status.as_str() {
            "finished" => *include_finished.read(),
            "reading" => *include_reading.read(),
            "want_to_read" => *include_want_to_read.read(),
            _ => true,
        })
        .count();
    let dupes = *duplicate_count.read();

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4",
            div {
                class: "flex max-h-[80vh] w-full max-w-lg flex-col rounded-xl
                    border border-gray-700 bg-gray-900 shadow-2xl",

                // Header
                div {
                    class: "flex items-center justify-between border-b border-gray-700 px-5 py-4",
                    h2 { class: "text-lg font-semibold text-gray-100", "Import from Goodreads" }
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
                            class: "mb-4 rounded-lg bg-red-900/40 px-4 py-2 text-sm text-red-300",
                            "{err}"
                        }
                    }

                    match *phase.read() {
                        Phase::FileSelect => rsx! {
                            div {
                                class: "flex flex-col items-center gap-4 py-8 text-center",
                                svg {
                                    class: "h-16 w-16 text-gray-500",
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
                                p { class: "text-gray-400", "Select your Goodreads library export CSV file" }
                                button {
                                    r#type: "button",
                                    onclick: handle_pick_file,
                                    class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                        text-white transition hover:bg-amber-700 active:scale-95",
                                    "Choose CSV File"
                                }
                            }
                        },
                        Phase::Parsing => rsx! {
                            div {
                                class: "flex flex-col items-center gap-3 py-12",
                                div { class: "h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" }
                                p { class: "text-sm text-gray-400", "Parsing CSV file..." }
                            }
                        },
                        Phase::Confirm => rsx! {
                            div {
                                class: "flex flex-col items-center gap-4 py-8 text-center",
                                div {
                                    class: "rounded-full bg-amber-900/40 p-3",
                                    svg {
                                        class: "h-8 w-8 text-amber-400",
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
                                p {
                                    class: "text-gray-300",
                                    "Found "
                                    span { class: "font-bold text-amber-400", "{total}" }
                                    if total == 1 { " book" } else { " books" }
                                    if filtered_count != total {
                                        " ("
                                        span { class: "font-bold text-amber-400", "{filtered_count}" }
                                        " selected)"
                                    }
                                }
                                if dupes > 0 {
                                    p {
                                        class: "text-sm text-gray-400",
                                        span { class: "font-bold text-amber-400", "{dupes}" }
                                        " already in library, will be skipped"
                                    }
                                }

                                // Import options
                                div {
                                    class: "mt-4 w-full border-t border-gray-700 pt-4 text-left space-y-3",

                                    // Shelf filter
                                    div {
                                        class: "space-y-1",
                                        p { class: "text-sm font-medium text-gray-300", "Shelves to import:" }
                                        label {
                                            class: "flex items-center gap-2 text-sm text-gray-400",
                                            input {
                                                r#type: "checkbox",
                                                class: "h-4 w-4 rounded border-gray-600 bg-gray-800 text-amber-500 accent-amber-500",
                                                checked: *include_finished.read(),
                                                onchange: move |_| { let v = *include_finished.read(); include_finished.set(!v); },
                                            }
                                            "Read"
                                        }
                                        label {
                                            class: "flex items-center gap-2 text-sm text-gray-400",
                                            input {
                                                r#type: "checkbox",
                                                class: "h-4 w-4 rounded border-gray-600 bg-gray-800 text-amber-500 accent-amber-500",
                                                checked: *include_reading.read(),
                                                onchange: move |_| { let v = *include_reading.read(); include_reading.set(!v); },
                                            }
                                            "Currently Reading"
                                        }
                                        label {
                                            class: "flex items-center gap-2 text-sm text-gray-400",
                                            input {
                                                r#type: "checkbox",
                                                class: "h-4 w-4 rounded border-gray-600 bg-gray-800 text-amber-500 accent-amber-500",
                                                checked: *include_want_to_read.read(),
                                                onchange: move |_| { let v = *include_want_to_read.read(); include_want_to_read.set(!v); },
                                            }
                                            "Want to Read"
                                        }
                                    }

                                    // Data options
                                    div {
                                        class: "space-y-1",
                                        label {
                                            class: "flex items-center gap-2 text-sm text-gray-400",
                                            input {
                                                r#type: "checkbox",
                                                class: "h-4 w-4 rounded border-gray-600 bg-gray-800 text-amber-500 accent-amber-500",
                                                checked: *import_ratings.read(),
                                                onchange: move |_| { let v = *import_ratings.read(); import_ratings.set(!v); },
                                            }
                                            "Import ratings"
                                        }
                                        label {
                                            class: "flex items-center gap-2 text-sm text-gray-400",
                                            input {
                                                r#type: "checkbox",
                                                class: "h-4 w-4 rounded border-gray-600 bg-gray-800 text-amber-500 accent-amber-500",
                                                checked: *import_read_dates.read(),
                                                onchange: move |_| { let v = *import_read_dates.read(); import_read_dates.set(!v); },
                                            }
                                            "Import read dates"
                                        }
                                        label {
                                            class: "flex items-center gap-2 text-sm text-gray-400",
                                            input {
                                                r#type: "checkbox",
                                                class: "h-4 w-4 rounded border-gray-600 bg-gray-800 text-amber-500 accent-amber-500",
                                                checked: *enrich.read(),
                                                onchange: move |_| { let v = *enrich.read(); enrich.set(!v); },
                                            }
                                            "Fetch covers & metadata"
                                        }
                                    }
                                }

                                div {
                                    class: "flex gap-3 mt-4",
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            parsed_books.set(Vec::new());
                                            phase.set(Phase::FileSelect);
                                        },
                                        class: "rounded-lg border border-gray-600 px-5 py-2 text-sm font-medium
                                            text-gray-300 transition hover:bg-gray-800 active:scale-95",
                                        "Cancel"
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: handle_import,
                                        class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                            text-white transition hover:bg-amber-700 active:scale-95",
                                        "Import"
                                    }
                                }
                            }
                        },
                        Phase::Importing => rsx! {
                            div {
                                class: "flex flex-col items-center gap-3 py-12",
                                div { class: "h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" }
                                p { class: "text-sm text-gray-400", "Importing books..." }
                            }
                        },
                        Phase::Done => {
                            let ic = *imported_count.read();
                            let sc = *skipped_count.read();
                            rsx! {
                                div {
                                    class: "flex flex-col items-center gap-4 py-8 text-center",
                                    div {
                                        class: "rounded-full bg-green-900/40 p-3",
                                        svg {
                                            class: "h-8 w-8 text-green-400",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M5 13l4 4L19 7",
                                            }
                                        }
                                    }
                                    p {
                                        class: "text-gray-300",
                                        "Imported "
                                        span { class: "font-bold text-amber-400", "{ic}" }
                                        if ic == 1 { " book" } else { " books" }
                                    }
                                    if sc > 0 {
                                        p {
                                            class: "text-sm text-gray-400",
                                            span { class: "font-bold text-amber-400", "{sc}" }
                                            " skipped (already in library)"
                                        }
                                    }
                                    if *enrichment_warning.read() {
                                        p {
                                            class: "text-sm text-amber-400",
                                            "Enrichment is already running. New books will be enriched when the current batch finishes."
                                        }
                                    } else if *enrich.read() && ic > 0 {
                                        p {
                                            class: "text-sm text-gray-400",
                                            "Enrichment running in background"
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            props.on_close.call(());
                                        },
                                        class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                            text-white transition hover:bg-amber-700 active:scale-95",
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
}
