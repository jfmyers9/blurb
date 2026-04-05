use dioxus::prelude::*;

use tracing::error;

use crate::data::commands::{
    check_clippings_exist, enrich_book_db, get_book_db, import_clippings_db, import_kindle_books_db,
};
use crate::services::kindle::{detect_kindle, list_kindle_books, KindleBook};
use crate::services::metadata;
use crate::DatabaseHandle;

#[derive(Clone, PartialEq)]
enum Phase {
    Disconnected,
    Detecting,
    Connected,
    Scanning,
    Results,
    Importing,
    Enriching,
    Clippings,
    ImportingClippings,
    Done,
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

fn ext_color(ext: &str) -> &'static str {
    match ext {
        "mobi" => "bg-blue-600",
        "azw" => "bg-purple-600",
        "azw3" => "bg-purple-500",
        "pdf" => "bg-red-600",
        "kfx" => "bg-green-600",
        _ => "bg-gray-600",
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct KindleSyncProps {
    on_close: EventHandler<()>,
    on_import_complete: EventHandler<()>,
}

#[component]
pub fn KindleSync(props: KindleSyncProps) -> Element {
    let db = use_context::<DatabaseHandle>();

    let mut phase = use_signal(|| Phase::Disconnected);
    let mut mount_path: Signal<Option<String>> = use_signal(|| None);
    let mut kindle_books: Signal<Vec<KindleBook>> = use_signal(Vec::new);
    let mut selected: Signal<Vec<bool>> = use_signal(Vec::new);
    let mut imported_count = use_signal(|| 0usize);
    let mut clippings_count = use_signal(|| 0usize);
    let mut imported_clippings_count = use_signal(|| 0usize);
    let mut enriching_progress: Signal<(usize, usize)> = use_signal(|| (0, 0));
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let handle_detect = move |_| {
        phase.set(Phase::Detecting);
        error.set(None);
        spawn(async move {
            match detect_kindle() {
                Some(path) => {
                    mount_path.set(Some(path));
                    phase.set(Phase::Connected);
                }
                None => {
                    phase.set(Phase::Disconnected);
                    error.set(Some(
                        "No Kindle device found. Make sure it's connected via USB.".into(),
                    ));
                }
            }
        });
    };

    let handle_scan = move |_| {
        let mp = mount_path.read().clone();
        let Some(mp) = mp else { return };
        phase.set(Phase::Scanning);
        error.set(None);
        spawn(async move {
            let books = list_kindle_books(&mp);
            let sel = vec![true; books.len()];
            kindle_books.set(books);
            selected.set(sel);
            phase.set(Phase::Results);
        });
    };

    let handle_import = {
        let db = db.clone();
        let on_import_complete = props.on_import_complete;
        move |_| {
            let db = db.clone();
            let books = kindle_books.read().clone();
            let sel = selected.read().clone();
            let mp = mount_path.read().clone();
            let to_import: Vec<KindleBook> = books
                .into_iter()
                .zip(sel.iter())
                .filter(|(_, s)| **s)
                .map(|(b, _)| b)
                .collect();
            if to_import.is_empty() {
                return;
            }
            phase.set(Phase::Importing);
            error.set(None);
            spawn(async move {
                let covers_dir = crate::data::db::covers_dir().ok();
                let result = {
                    let mut conn = db.conn.lock().unwrap();
                    import_kindle_books_db(&mut conn, &to_import, covers_dir.as_deref())
                };
                match result {
                    Ok(ids) => {
                        imported_count.set(ids.len());
                        on_import_complete.call(());

                        // Check which imported books need enrichment
                        let needs_enrichment: Vec<_> = {
                            let conn = db.conn.lock().unwrap();
                            ids.iter()
                                .filter_map(|&id| get_book_db(&conn, id).ok())
                                .filter(|b| b.author.is_none() || b.cover_url.is_none())
                                .collect()
                        };

                        if !needs_enrichment.is_empty() {
                            let total = needs_enrichment.len();
                            enriching_progress.set((0, total));
                            phase.set(Phase::Enriching);

                            for (i, book) in needs_enrichment.into_iter().enumerate() {
                                enriching_progress.set((i + 1, total));
                                match metadata::search_by_title(&book.title, book.author.as_deref())
                                    .await
                                {
                                    Ok(meta) => {
                                        let conn = db.conn.lock().unwrap();
                                        if let Err(e) = enrich_book_db(&conn, book.id, &meta) {
                                            error!("enrich book {}: {e}", book.id);
                                        }
                                    }
                                    Err(e) => {
                                        error!("metadata fetch for book {}: {e}", book.id);
                                    }
                                }
                                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                            }
                            on_import_complete.call(());
                        }

                        if let Some(ref mp) = mp {
                            if let Ok(info) = check_clippings_exist(mp) {
                                if info.exists && info.count > 0 {
                                    clippings_count.set(info.count);
                                    phase.set(Phase::Clippings);
                                    return;
                                }
                            }
                        }
                        phase.set(Phase::Done);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        phase.set(Phase::Results);
                    }
                }
            });
        }
    };

    let handle_import_clippings = {
        let db = db.clone();
        move |_| {
            let db = db.clone();
            let mp = mount_path.read().clone();
            let Some(mp) = mp else { return };
            phase.set(Phase::ImportingClippings);
            error.set(None);
            spawn(async move {
                let result = {
                    let mut conn = db.conn.lock().unwrap();
                    import_clippings_db(&mut conn, &mp)
                };
                match result {
                    Ok(count) => imported_clippings_count.set(count),
                    Err(e) => error.set(Some(e)),
                }
                phase.set(Phase::Done);
            });
        }
    };

    let selected_count = selected.read().iter().filter(|s| **s).count();

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4",
            div {
                class: "flex max-h-[80vh] w-full max-w-lg flex-col rounded-xl
                    border border-gray-700 bg-gray-900 shadow-2xl",

                // Header
                div {
                    class: "flex items-center justify-between border-b border-gray-700 px-5 py-4",
                    h2 { class: "text-lg font-semibold text-gray-100", "Kindle Sync" }
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
                        Phase::Disconnected => rsx! {
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
                                        d: "M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25",
                                    }
                                }
                                p { class: "text-gray-400", "Connect your Kindle via USB to import books" }
                                button {
                                    r#type: "button",
                                    onclick: handle_detect,
                                    class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                        text-white transition hover:bg-amber-700 active:scale-95",
                                    "Check Connection"
                                }
                            }
                        },
                        Phase::Detecting => rsx! {
                            div {
                                class: "flex flex-col items-center gap-3 py-12",
                                div { class: "h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" }
                                p { class: "text-sm text-gray-400", "Scanning for Kindle..." }
                            }
                        },
                        Phase::Connected => rsx! {
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
                                    "Kindle detected at "
                                    span { class: "font-mono text-amber-400",
                                        "{mount_path.read().as_deref().unwrap_or(\"\")}"
                                    }
                                }
                                button {
                                    r#type: "button",
                                    onclick: handle_scan,
                                    class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                        text-white transition hover:bg-amber-700 active:scale-95",
                                    "Scan Books"
                                }
                            }
                        },
                        Phase::Scanning => rsx! {
                            div {
                                class: "flex flex-col items-center gap-3 py-12",
                                div { class: "h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" }
                                p { class: "text-sm text-gray-400", "Scanning Kindle library..." }
                            }
                        },
                        Phase::Results => {
                            let books = kindle_books.read();
                            if books.is_empty() {
                                rsx! {
                                    p { class: "py-8 text-center text-gray-400", "No supported books found on Kindle." }
                                }
                            } else {
                                let count = books.len();
                                let all_selected = selected.read().iter().all(|s| *s);
                                rsx! {
                                    div {
                                        class: "flex flex-col gap-3",
                                        div {
                                            class: "flex items-center justify-between",
                                            span {
                                                class: "text-sm text-gray-400",
                                                "{count} book{plural(count)} found"
                                            }
                                            button {
                                                r#type: "button",
                                                onclick: move |_| {
                                                    let all = selected.read().iter().all(|s| *s);
                                                    let len = selected.read().len();
                                                    selected.set(vec![!all; len]);
                                                },
                                                class: "text-xs text-amber-400 hover:text-amber-300",
                                                if all_selected { "Deselect all" } else { "Select all" }
                                            }
                                        }
                                        div {
                                            class: "flex flex-col gap-1",
                                            for (idx, book) in books.iter().enumerate() {
                                                {
                                                    let is_checked = selected.read().get(idx).copied().unwrap_or(false);
                                                    let title = book.title.clone();
                                                    let author = book.author.clone();
                                                    let publisher = book.publisher.clone();
                                                    let has_cover = book.cover_data.is_some();
                                                    let cover_b64 = book.cover_data.clone().unwrap_or_default();
                                                    let ext = book.extension.clone();
                                                    let cde_type = book.cde_type.clone();
                                                    let size = format_bytes(book.size_bytes);
                                                    let color = ext_color(&ext);
                                                    rsx! {
                                                        label {
                                                            class: "flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 transition hover:bg-gray-800",
                                                            input {
                                                                r#type: "checkbox",
                                                                checked: is_checked,
                                                                onchange: move |_| {
                                                                    let mut s = selected.read().clone();
                                                                    if let Some(v) = s.get_mut(idx) {
                                                                        *v = !*v;
                                                                    }
                                                                    selected.set(s);
                                                                },
                                                                class: "h-4 w-4 shrink-0 rounded border-gray-600 bg-gray-800 text-amber-500 accent-amber-500",
                                                            }
                                                            if has_cover {
                                                                img {
                                                                    src: "data:image/jpeg;base64,{cover_b64}",
                                                                    alt: "",
                                                                    class: "h-12 w-9 shrink-0 rounded object-cover",
                                                                }
                                                            } else {
                                                                div {
                                                                    class: "flex h-12 w-9 shrink-0 items-center justify-center rounded bg-gray-700 text-[10px] text-gray-500",
                                                                    "No cover"
                                                                }
                                                            }
                                                            div {
                                                                class: "min-w-0 flex-1",
                                                                p { class: "truncate text-sm font-medium text-gray-200", "{title}" }
                                                                if let Some(ref a) = author {
                                                                    p { class: "truncate text-xs text-gray-500", "{a}" }
                                                                }
                                                                if let Some(ref pb) = publisher {
                                                                    p { class: "truncate text-xs text-gray-600", "{pb}" }
                                                                }
                                                            }
                                                            div {
                                                                class: "flex shrink-0 flex-col items-end gap-1",
                                                                div {
                                                                    class: "flex gap-1",
                                                                    if let Some(ref ct) = cde_type {
                                                                        span {
                                                                            class: "rounded bg-gray-700 px-1.5 py-0.5 text-[10px] font-medium uppercase text-gray-300",
                                                                            "{ct}"
                                                                        }
                                                                    }
                                                                    span {
                                                                        class: "rounded px-1.5 py-0.5 text-[10px] font-bold uppercase text-white {color}",
                                                                        "{ext}"
                                                                    }
                                                                }
                                                                span { class: "text-xs text-gray-500", "{size}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
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
                        Phase::Enriching => {
                            let (current, total) = *enriching_progress.read();
                            rsx! {
                                div {
                                    class: "flex flex-col items-center gap-3 py-12",
                                    div { class: "h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" }
                                    p { class: "text-sm text-gray-400",
                                        "Enriching metadata... ({current}/{total})"
                                    }
                                }
                            }
                        },
                        Phase::Clippings => {
                            let ic = *imported_count.read();
                            let cc = *clippings_count.read();
                            rsx! {
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
                                                d: "M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z",
                                            }
                                        }
                                    }
                                    p {
                                        class: "text-gray-300",
                                        "Imported "
                                        span { class: "font-bold text-amber-400", "{ic}" }
                                        " book{plural(ic)}"
                                    }
                                    p {
                                        class: "text-sm text-gray-400",
                                        "Found "
                                        span { class: "font-bold text-amber-400", "{cc}" }
                                        " highlight{plural(cc)} in My Clippings.txt"
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: handle_import_clippings,
                                        class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                            text-white transition hover:bg-amber-700 active:scale-95",
                                        "Import Highlights"
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: move |_| phase.set(Phase::Done),
                                        class: "text-sm text-gray-500 hover:text-gray-300",
                                        "Skip"
                                    }
                                }
                            }
                        },
                        Phase::ImportingClippings => rsx! {
                            div {
                                class: "flex flex-col items-center gap-3 py-12",
                                div { class: "h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" }
                                p { class: "text-sm text-gray-400", "Importing highlights..." }
                            }
                        },
                        Phase::Done => {
                            let ic = *imported_count.read();
                            let icc = *imported_clippings_count.read();
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
                                        " book{plural(ic)}"
                                    }
                                    if icc > 0 {
                                        p {
                                            class: "text-gray-300",
                                            "Imported "
                                            span { class: "font-bold text-amber-400", "{icc}" }
                                            " highlight{plural(icc)}"
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: move |_| props.on_close.call(()),
                                        class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                                            text-white transition hover:bg-amber-700 active:scale-95",
                                        "Close"
                                    }
                                }
                            }
                        },
                    }
                }

                // Footer for results phase
                if *phase.read() == Phase::Results && !kindle_books.read().is_empty() && selected_count > 0 {
                    div {
                        class: "border-t border-gray-700 px-5 py-3",
                        button {
                            r#type: "button",
                            onclick: handle_import,
                            class: "w-full rounded-lg bg-amber-600 py-2 text-sm font-medium
                                text-white transition hover:bg-amber-700 active:scale-95",
                            "Import {selected_count} Book{plural(selected_count)}"
                        }
                    }
                }
            }
        }
    }
}
