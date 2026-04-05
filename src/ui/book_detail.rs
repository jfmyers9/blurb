use dioxus::prelude::*;
use tracing::error;

use crate::data::commands::{
    add_book_to_shelf_db, create_shelf_db, delete_book_with_covers_db, enrich_book_db, get_book_db,
    list_book_diary_entries_db, list_book_shelves_db, list_highlights_db,
    remove_book_from_shelf_db, set_rating_db, set_reading_status_db, update_book_db,
    update_reading_dates_db, upload_cover_db,
};
use crate::data::models::{Book, DiaryEntry, Highlight, Shelf};
use crate::services::metadata;
use crate::DatabaseHandle;

use super::diary_entry_form::DiaryEntryForm;
use super::rating_stars::RatingStars;
use super::shelf_picker::ShelfPicker;
use super::status_select::StatusSelect;

#[derive(Props, Clone, PartialEq)]
pub struct BookDetailProps {
    book_id: i64,
    shelves: Vec<Shelf>,
    on_close: EventHandler<()>,
    on_changed: EventHandler<()>,
    on_deleted: EventHandler<i64>,
}

#[component]
pub fn BookDetail(props: BookDetailProps) -> Element {
    let db = use_context::<DatabaseHandle>();
    let book_id = props.book_id;

    let mut book: Signal<Option<Book>> = use_signal(|| None);
    let mut highlights: Signal<Vec<Highlight>> = use_signal(Vec::new);
    let mut diary_entries: Signal<Vec<DiaryEntry>> = use_signal(Vec::new);
    let mut book_shelf_ids: Signal<Vec<i64>> = use_signal(Vec::new);

    let mut title = use_signal(String::new);
    let mut author = use_signal(String::new);
    let mut confirm_delete = use_signal(|| false);
    let mut enriching = use_signal(|| false);

    let mut show_diary_form = use_signal(|| false);

    // Cover editing state
    let mut show_cover_menu = use_signal(|| false);
    let mut cover_mode = use_signal(|| None::<CoverMode>);
    let mut paste_url = use_signal(String::new);
    let mut search_query = use_signal(String::new);
    let mut search_results = use_signal(Vec::<metadata::BookMetadata>::new);
    let mut searching = use_signal(|| false);

    // Load book data
    {
        let db = db.clone();
        use_effect(move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                match get_book_db(&conn, book_id) {
                    Ok(b) => {
                        let q = format!("{} {}", b.title, b.author.as_deref().unwrap_or(""))
                            .trim()
                            .to_string();
                        title.set(b.title.clone());
                        author.set(b.author.clone().unwrap_or_default());
                        search_query.set(q);
                        book.set(Some(b));
                    }
                    Err(e) => error!("failed to load book {book_id}: {e}"),
                }
                match list_highlights_db(&conn, book_id) {
                    Ok(h) => highlights.set(h),
                    Err(e) => error!("failed to load highlights for book {book_id}: {e}"),
                }
                match list_book_shelves_db(&conn, book_id) {
                    Ok(shelves) => {
                        book_shelf_ids.set(shelves.iter().map(|s| s.id).collect());
                    }
                    Err(e) => error!("failed to load shelves for book {book_id}: {e}"),
                }
                match list_book_diary_entries_db(&conn, book_id) {
                    Ok(entries) => diary_entries.set(entries),
                    Err(e) => error!("failed to load diary entries for book {book_id}: {e}"),
                }
            });
        });
    }

    let refresh_book = {
        let db = db.clone();
        let on_changed = props.on_changed;
        move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                match get_book_db(&conn, book_id) {
                    Ok(b) => {
                        title.set(b.title.clone());
                        author.set(b.author.clone().unwrap_or_default());
                        book.set(Some(b));
                    }
                    Err(e) => error!("failed to refresh book {book_id}: {e}"),
                }
                drop(conn);
                on_changed.call(());
            });
        }
    };

    let Some(bk) = book.read().clone() else {
        return rsx! {
            div { class: "fixed inset-0 z-40 bg-black/30" }
            div {
                class: "fixed top-0 right-0 z-50 flex h-full w-full max-w-md items-center justify-center bg-white dark:bg-gray-900",
                p { class: "text-gray-500", "Loading..." }
            }
        };
    };

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 z-40 bg-black/30",
            onclick: move |_| props.on_close.call(()),
        }
        // Panel
        div {
            class: "fixed top-0 right-0 z-50 flex h-full w-full max-w-md
                flex-col overflow-y-auto bg-white shadow-xl dark:bg-gray-900
                animate-slide-in",

            // Header
            div {
                class: "flex items-center justify-between border-b border-gray-200 px-4 py-3 dark:border-gray-700",
                h2 {
                    class: "text-sm font-medium text-gray-500 dark:text-gray-400",
                    "Book Details"
                }
                button {
                    r#type: "button",
                    onclick: move |_| props.on_close.call(()),
                    class: "rounded-md p-1 text-gray-400 hover:text-gray-600
                        dark:hover:text-gray-200",
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

            div {
                class: "flex-1 space-y-5 p-5",

                // Cover
                div {
                    class: "group relative mx-auto aspect-[2/3] w-48 overflow-hidden rounded-lg bg-gray-100 dark:bg-gray-700",
                    onmouseenter: move |_| show_cover_menu.set(true),
                    onmouseleave: move |_| {
                        if cover_mode.read().is_none() {
                            show_cover_menu.set(false);
                        }
                    },
                    if let Some(ref cover_url) = bk.cover_url {
                        img {
                            src: "{cover_url}",
                            alt: "{bk.title}",
                            class: "h-full w-full object-cover",
                        }
                    } else {
                        div {
                            class: "flex h-full w-full items-center justify-center
                                bg-gradient-to-br from-amber-100 to-orange-200
                                dark:from-amber-900/40 dark:to-orange-900/40",
                            span {
                                class: "text-6xl font-bold text-amber-700/60 dark:text-amber-400/60",
                                "{bk.title.chars().next().unwrap_or('?').to_uppercase()}"
                            }
                        }
                    }
                    // Edit overlay
                    if *show_cover_menu.read() || cover_mode.read().is_some() {
                        div {
                            class: "absolute inset-0 flex items-end bg-black/40",
                            if cover_mode.read().is_none() {
                                div {
                                    class: "flex w-full flex-col gap-1 p-2",
                                    button {
                                        r#type: "button",
                                        onclick: move |_| cover_mode.set(Some(CoverMode::Search)),
                                        class: "rounded bg-white/90 px-2 py-1 text-xs font-medium text-gray-800 hover:bg-white",
                                        "Search cover"
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: move |_| cover_mode.set(Some(CoverMode::Paste)),
                                        class: "rounded bg-white/90 px-2 py-1 text-xs font-medium text-gray-800 hover:bg-white",
                                        "Paste URL"
                                    }
                                    button {
                                        r#type: "button",
                                        onclick: {
                                            let db = db.clone();
                                            let refresh_book = refresh_book.clone();
                                            move |_| {
                                                let db = db.clone();
                                                let refresh_book = refresh_book.clone();
                                                spawn(async move {
                                                    let file = rfd::AsyncFileDialog::new()
                                                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                                                        .pick_file()
                                                        .await;
                                                    if let Some(file) = file {
                                                        let path = file.path().to_string_lossy().to_string();
                                                        let conn = db.conn.lock().unwrap();
                                                        if let Err(e) = upload_cover_db(&conn, book_id, &path) {
                                                            error!("failed to upload cover for book {book_id}: {e}");
                                                        }
                                                        drop(conn);
                                                        cover_mode.set(None);
                                                        refresh_book();
                                                    }
                                                });
                                            }
                                        },
                                        class: "rounded bg-white/90 px-2 py-1 text-xs font-medium text-gray-800 hover:bg-white",
                                        "Upload file"
                                    }
                                }
                            }
                        }
                    }
                }

                // Paste URL input
                if matches!(*cover_mode.read(), Some(CoverMode::Paste)) {
                    div {
                        class: "mx-auto flex w-48 flex-col gap-2",
                        input {
                            r#type: "url",
                            value: "{paste_url}",
                            oninput: move |evt: Event<FormData>| paste_url.set(evt.value()),
                            placeholder: "https://...",
                            autofocus: true,
                            class: "w-full rounded-md border border-gray-300 bg-white px-2 py-1.5
                                text-xs text-gray-900 dark:border-gray-600 dark:bg-gray-800
                                dark:text-gray-100 focus:ring-2 focus:ring-amber-500 focus:outline-none",
                            onkeydown: {
                                let db = db.clone();
                                let refresh_book = refresh_book.clone();
                                let bk = bk.clone();
                                move |evt: Event<KeyboardData>| {
                                    if evt.key() == Key::Enter {
                                        let url = paste_url.read().trim().to_string();
                                        if !url.is_empty() {
                                            let db = db.clone();
                                            let refresh_book = refresh_book.clone();
                                            let bk = bk.clone();
                                            let t = title.read().clone();
                                            let a = author.read().clone();
                                            spawn(async move {
                                                let conn = db.conn.lock().unwrap();
                                                if let Err(e) = update_book_db(
                                                    &conn, book_id, &t,
                                                    if a.is_empty() { None } else { Some(a.as_str()) },
                                                    bk.isbn.as_deref(), bk.asin.as_deref(),
                                                    Some(url.as_str()),
                                                    bk.description.as_deref(), bk.publisher.as_deref(),
                                                    bk.published_date.as_deref(), bk.page_count,
                                                ) {
                                                    error!("failed to update cover URL for book {book_id}: {e}");
                                                }
                                                drop(conn);
                                                cover_mode.set(None);
                                                paste_url.set(String::new());
                                                refresh_book();
                                            });
                                        }
                                    }
                                }
                            },
                        }
                        div {
                            class: "flex gap-2",
                            button {
                                r#type: "button",
                                onclick: {
                                    let db = db.clone();
                                    let refresh_book = refresh_book.clone();
                                    let bk = bk.clone();
                                    move |_| {
                                        let url = paste_url.read().trim().to_string();
                                        if !url.is_empty() {
                                            let db = db.clone();
                                            let refresh_book = refresh_book.clone();
                                            let bk = bk.clone();
                                            let t = title.read().clone();
                                            let a = author.read().clone();
                                            spawn(async move {
                                                let conn = db.conn.lock().unwrap();
                                                if let Err(e) = update_book_db(
                                                    &conn, book_id, &t,
                                                    if a.is_empty() { None } else { Some(a.as_str()) },
                                                    bk.isbn.as_deref(), bk.asin.as_deref(),
                                                    Some(url.as_str()),
                                                    bk.description.as_deref(), bk.publisher.as_deref(),
                                                    bk.published_date.as_deref(), bk.page_count,
                                                ) {
                                                    error!("failed to update cover URL for book {book_id}: {e}");
                                                }
                                                drop(conn);
                                                cover_mode.set(None);
                                                paste_url.set(String::new());
                                                refresh_book();
                                            });
                                        }
                                    }
                                },
                                class: "flex-1 rounded-md bg-amber-600 px-2 py-1 text-xs font-medium text-white hover:bg-amber-700",
                                "Apply"
                            }
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    cover_mode.set(None);
                                    paste_url.set(String::new());
                                },
                                class: "flex-1 rounded-md border border-gray-300 px-2 py-1 text-xs font-medium
                                    text-gray-600 hover:bg-gray-50 dark:border-gray-600
                                    dark:text-gray-400 dark:hover:bg-gray-800",
                                "Cancel"
                            }
                        }
                    }
                }

                // Search cover
                if matches!(*cover_mode.read(), Some(CoverMode::Search)) {
                    div {
                        class: "space-y-2",
                        div {
                            class: "flex gap-2",
                            input {
                                r#type: "text",
                                value: "{search_query}",
                                oninput: move |evt: Event<FormData>| search_query.set(evt.value()),
                                autofocus: true,
                                class: "min-w-0 flex-1 rounded-md border border-gray-300 bg-white px-2 py-1.5
                                    text-xs text-gray-900 dark:border-gray-600 dark:bg-gray-800
                                    dark:text-gray-100 focus:ring-2 focus:ring-amber-500 focus:outline-none",
                                onkeydown: {
                                    move |evt: Event<KeyboardData>| {
                                        if evt.key() == Key::Enter {
                                            let query = search_query.read().clone();
                                            if !query.trim().is_empty() {
                                                searching.set(true);
                                                spawn(async move {
                                                    match metadata::search_covers(&query).await {
                                                        Ok(results) => search_results.set(results),
                                                        Err(e) => {
                                                            error!("failed to search covers: {e}");
                                                            search_results.set(vec![]);
                                                        }
                                                    }
                                                    searching.set(false);
                                                });
                                            }
                                        }
                                    }
                                },
                            }
                            button {
                                r#type: "button",
                                onclick: {
                                    move |_| {
                                        let query = search_query.read().clone();
                                        if !query.trim().is_empty() {
                                            searching.set(true);
                                            spawn(async move {
                                                match metadata::search_covers(&query).await {
                                                    Ok(results) => search_results.set(results),
                                                    Err(e) => {
                                                        error!("failed to search covers: {e}");
                                                        search_results.set(vec![]);
                                                    }
                                                }
                                                searching.set(false);
                                            });
                                        }
                                    }
                                },
                                disabled: *searching.read(),
                                class: "rounded-md bg-amber-600 px-3 py-1 text-xs font-medium text-white
                                    hover:bg-amber-700 disabled:opacity-50",
                                if *searching.read() { "..." } else { "Search" }
                            }
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    cover_mode.set(None);
                                    search_results.set(vec![]);
                                },
                                class: "rounded-md border border-gray-300 px-2 py-1 text-xs font-medium
                                    text-gray-600 hover:bg-gray-50 dark:border-gray-600
                                    dark:text-gray-400 dark:hover:bg-gray-800",
                                "Cancel"
                            }
                        }
                        if *searching.read() {
                            p { class: "text-center text-xs text-gray-500", "Searching..." }
                        }
                        if !search_results.read().is_empty() {
                            div {
                                class: "grid grid-cols-3 gap-2",
                                for (i, result) in search_results.read().iter().enumerate() {
                                    {
                                        let cover_url = result.cover_url.clone();
                                        let result_title = result.title.clone().unwrap_or_default();
                                        let result_author = result.author.clone();
                                        rsx! {
                                            button {
                                                key: "{i}",
                                                r#type: "button",
                                                onclick: {
                                                    let cover_url = cover_url.clone();
                                                    let db = db.clone();
                                                    let refresh_book = refresh_book.clone();
                                                    let bk = bk.clone();
                                                    move |_| {
                                                        if let Some(ref url) = cover_url {
                                                            let url = url.clone();
                                                            let db = db.clone();
                                                            let refresh_book = refresh_book.clone();
                                                            let bk = bk.clone();
                                                            let t = title.read().clone();
                                                            let a = author.read().clone();
                                                            spawn(async move {
                                                                let conn = db.conn.lock().unwrap();
                                                                if let Err(e) = update_book_db(
                                                                    &conn, book_id, &t,
                                                                    if a.is_empty() { None } else { Some(a.as_str()) },
                                                                    bk.isbn.as_deref(), bk.asin.as_deref(),
                                                                    Some(url.as_str()),
                                                                    bk.description.as_deref(), bk.publisher.as_deref(),
                                                                    bk.published_date.as_deref(), bk.page_count,
                                                                ) {
                                                                    error!("failed to update cover from search for book {book_id}: {e}");
                                                                }
                                                                drop(conn);
                                                                cover_mode.set(None);
                                                                search_results.set(vec![]);
                                                                refresh_book();
                                                            });
                                                        }
                                                    }
                                                },
                                                disabled: cover_url.is_none(),
                                                class: "group/thumb flex flex-col gap-1 rounded-md border border-gray-200
                                                    p-1 text-left hover:border-amber-400 hover:bg-amber-50
                                                    dark:border-gray-700 dark:hover:border-amber-600
                                                    dark:hover:bg-amber-900/20 disabled:opacity-40",
                                                if let Some(ref url) = cover_url {
                                                    img {
                                                        src: "{url}",
                                                        alt: "{result_title}",
                                                        class: "aspect-[2/3] w-full rounded object-cover",
                                                    }
                                                } else {
                                                    div {
                                                        class: "flex aspect-[2/3] w-full items-center justify-center rounded bg-gray-100 dark:bg-gray-700",
                                                        span { class: "text-xs text-gray-400", "No img" }
                                                    }
                                                }
                                                span {
                                                    class: "line-clamp-2 text-[10px] leading-tight text-gray-700 dark:text-gray-300",
                                                    "{result_title}"
                                                }
                                                if let Some(ref a) = result_author {
                                                    span {
                                                        class: "line-clamp-1 text-[10px] leading-tight text-gray-500 dark:text-gray-400",
                                                        "{a}"
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

                // Enrich metadata
                if bk.cover_url.is_none() || bk.description.is_none() {
                    div {
                        class: "flex justify-center",
                        button {
                            r#type: "button",
                            disabled: *enriching.read(),
                            onclick: {
                                let db = db.clone();
                                let refresh_book = refresh_book.clone();
                                let bk = bk.clone();
                                move |_| {
                                    enriching.set(true);
                                    let db = db.clone();
                                    let refresh_book = refresh_book.clone();
                                    let bk = bk.clone();
                                    spawn(async move {
                                        let meta = metadata::search_by_title(
                                            &bk.title,
                                            bk.author.as_deref(),
                                        ).await;
                                        match meta {
                                            Ok(meta) => {
                                                let conn = db.conn.lock().unwrap();
                                                if let Err(e) = enrich_book_db(&conn, book_id, &meta) {
                                                    error!("failed to enrich book {book_id}: {e}");
                                                }
                                                drop(conn);
                                            }
                                            Err(e) => error!("failed to fetch metadata for book {book_id}: {e}"),
                                        }
                                        enriching.set(false);
                                        refresh_book();
                                    });
                                }
                            },
                            class: "rounded-md bg-amber-600/10 px-3 py-1.5 text-xs font-medium
                                text-amber-600 hover:bg-amber-600/20
                                dark:text-amber-400 dark:hover:bg-amber-600/20
                                disabled:opacity-50 disabled:cursor-not-allowed",
                            if *enriching.read() { "Enriching..." } else { "Enrich metadata" }
                        }
                    }
                }

                // Title
                div {
                    label {
                        class: "mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400",
                        "Title"
                    }
                    input {
                        r#type: "text",
                        value: "{title}",
                        oninput: move |evt: Event<FormData>| title.set(evt.value()),
                        onblur: {
                            let db = db.clone();
                            let refresh_book = refresh_book.clone();
                            let bk = bk.clone();
                            move |_| {
                                let t = title.read().trim().to_string();
                                if !t.is_empty() && t != bk.title {
                                    let a = author.read().clone();
                                    let db = db.clone();
                                    let refresh_book = refresh_book.clone();
                                    let bk = bk.clone();
                                    spawn(async move {
                                        let conn = db.conn.lock().unwrap();
                                        if let Err(e) = update_book_db(
                                            &conn, book_id, &t,
                                            if a.is_empty() { None } else { Some(a.as_str()) },
                                            bk.isbn.as_deref(), bk.asin.as_deref(),
                                            bk.cover_url.as_deref(), bk.description.as_deref(),
                                            bk.publisher.as_deref(), bk.published_date.as_deref(),
                                            bk.page_count,
                                        ) {
                                            error!("failed to update title for book {book_id}: {e}");
                                        }
                                        drop(conn);
                                        refresh_book();
                                    });
                                }
                            }
                        },
                        class: "w-full rounded-md border border-gray-300 bg-white px-3
                            py-2 text-base font-semibold text-gray-900
                            dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
                            focus:ring-2 focus:ring-amber-500 focus:outline-none",
                    }
                }

                // Author
                div {
                    label {
                        class: "mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400",
                        "Author"
                    }
                    input {
                        r#type: "text",
                        value: "{author}",
                        oninput: move |evt: Event<FormData>| author.set(evt.value()),
                        onblur: {
                            let db = db.clone();
                            let refresh_book = refresh_book.clone();
                            let bk = bk.clone();
                            move |_| {
                                let a = author.read().clone();
                                let orig = bk.author.as_deref().unwrap_or("");
                                if a != orig {
                                    let t = title.read().clone();
                                    let t = if t.trim().is_empty() { bk.title.clone() } else { t.trim().to_string() };
                                    let db = db.clone();
                                    let refresh_book = refresh_book.clone();
                                    let bk = bk.clone();
                                    spawn(async move {
                                        let conn = db.conn.lock().unwrap();
                                        if let Err(e) = update_book_db(
                                            &conn, book_id, &t,
                                            if a.trim().is_empty() { None } else { Some(a.trim()) },
                                            bk.isbn.as_deref(), bk.asin.as_deref(),
                                            bk.cover_url.as_deref(), bk.description.as_deref(),
                                            bk.publisher.as_deref(), bk.published_date.as_deref(),
                                            bk.page_count,
                                        ) {
                                            error!("failed to update author for book {book_id}: {e}");
                                        }
                                        drop(conn);
                                        refresh_book();
                                    });
                                }
                            }
                        },
                        placeholder: "Unknown",
                        class: "w-full rounded-md border border-gray-300 bg-white px-3
                            py-2 text-sm text-gray-900 dark:border-gray-600
                            dark:bg-gray-800 dark:text-gray-100 focus:ring-2
                            focus:ring-amber-500 focus:outline-none",
                    }
                }

                // Rating
                div {
                    label {
                        class: "mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400",
                        "Rating"
                    }
                    RatingStars {
                        rating: bk.rating,
                        on_rate: {
                            let db = db.clone();
                            let refresh_book = refresh_book.clone();
                            move |score: i32| {
                                let db = db.clone();
                                let refresh_book = refresh_book.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    if let Err(e) = set_rating_db(&conn, book_id, score) {
                                        error!("failed to set rating for book {book_id}: {e}");
                                    }
                                    drop(conn);
                                    refresh_book();
                                });
                            }
                        },
                    }
                }

                // Status
                div {
                    label {
                        class: "mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400",
                        "Status"
                    }
                    StatusSelect {
                        status: bk.status.clone(),
                        on_change: {
                            let db = db.clone();
                            let refresh_book = refresh_book.clone();
                            move |status: String| {
                                let db = db.clone();
                                let refresh_book = refresh_book.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    if status.is_empty() {
                                        if let Err(e) = conn.execute(
                                            "DELETE FROM reading_status WHERE book_id = ?1",
                                            [book_id],
                                        ) {
                                            error!("failed to clear reading status for book {book_id}: {e}");
                                        }
                                    } else if let Err(e) = set_reading_status_db(&conn, book_id, &status, None, None) {
                                        error!("failed to set reading status for book {book_id}: {e}");
                                    }
                                    drop(conn);
                                    refresh_book();
                                });
                            }
                        },
                    }
                }

                // Reading Dates
                if bk.status.is_some() {
                    div {
                        class: "flex gap-4",
                        div {
                            class: "flex-1",
                            label {
                                class: "mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400",
                                "Started"
                            }
                            input {
                                r#type: "date",
                                value: "{bk.started_at.as_deref().unwrap_or(\"\")}",
                                onchange: {
                                    let db = db.clone();
                                    let refresh_book = refresh_book.clone();
                                    let finished = bk.finished_at.clone();
                                    move |evt: Event<FormData>| {
                                        let val = evt.value();
                                        let started = if val.is_empty() { None } else { Some(val) };
                                        let finished = finished.clone();
                                        let db = db.clone();
                                        let refresh_book = refresh_book.clone();
                                        spawn(async move {
                                            let conn = db.conn.lock().unwrap();
                                            if let Err(e) = update_reading_dates_db(
                                                &conn, book_id,
                                                started.as_deref(),
                                                finished.as_deref(),
                                            ) {
                                                error!("failed to update reading dates for book {book_id}: {e}");
                                            }
                                            drop(conn);
                                            refresh_book();
                                        });
                                    }
                                },
                                class: "w-full rounded-md border border-gray-300 bg-white px-2 py-1.5
                                    text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-800
                                    dark:text-gray-100 focus:ring-2 focus:ring-amber-500 focus:outline-none",
                            }
                        }
                        div {
                            class: "flex-1",
                            label {
                                class: "mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400",
                                if bk.status.as_deref() == Some("abandoned") { "Abandoned" } else { "Finished" }
                            }
                            input {
                                r#type: "date",
                                value: "{bk.finished_at.as_deref().unwrap_or(\"\")}",
                                onchange: {
                                    let db = db.clone();
                                    let refresh_book = refresh_book.clone();
                                    let started = bk.started_at.clone();
                                    move |evt: Event<FormData>| {
                                        let val = evt.value();
                                        let finished = if val.is_empty() { None } else { Some(val) };
                                        let started = started.clone();
                                        let db = db.clone();
                                        let refresh_book = refresh_book.clone();
                                        spawn(async move {
                                            let conn = db.conn.lock().unwrap();
                                            if let Err(e) = update_reading_dates_db(
                                                &conn, book_id,
                                                started.as_deref(),
                                                finished.as_deref(),
                                            ) {
                                                error!("failed to update reading dates for book {book_id}: {e}");
                                            }
                                            drop(conn);
                                            refresh_book();
                                        });
                                    }
                                },
                                class: "w-full rounded-md border border-gray-300 bg-white px-2 py-1.5
                                    text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-800
                                    dark:text-gray-100 focus:ring-2 focus:ring-amber-500 focus:outline-none",
                            }
                        }
                    }
                }

                // Shelves
                div {
                    label {
                        class: "mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400",
                        "Shelves"
                    }
                    ShelfPicker {
                        shelves: props.shelves.clone(),
                        book_shelf_ids: book_shelf_ids.read().clone(),
                        on_add: {
                            let db = db.clone();
                            let on_changed = props.on_changed;
                            move |shelf_id: i64| {
                                let db = db.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    if let Err(e) = add_book_to_shelf_db(&conn, book_id, shelf_id) {
                                        error!("failed to add book {book_id} to shelf {shelf_id}: {e}");
                                    }
                                    match list_book_shelves_db(&conn, book_id) {
                                        Ok(shelves) => book_shelf_ids.set(shelves.iter().map(|s| s.id).collect()),
                                        Err(e) => error!("failed to reload shelves for book {book_id}: {e}"),
                                    }
                                    drop(conn);
                                    on_changed.call(());
                                });
                            }
                        },
                        on_remove: {
                            let db = db.clone();
                            let on_changed = props.on_changed;
                            move |shelf_id: i64| {
                                let db = db.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    if let Err(e) = remove_book_from_shelf_db(&conn, book_id, shelf_id) {
                                        error!("failed to remove book {book_id} from shelf {shelf_id}: {e}");
                                    }
                                    match list_book_shelves_db(&conn, book_id) {
                                        Ok(shelves) => book_shelf_ids.set(shelves.iter().map(|s| s.id).collect()),
                                        Err(e) => error!("failed to reload shelves for book {book_id}: {e}"),
                                    }
                                    drop(conn);
                                    on_changed.call(());
                                });
                            }
                        },
                        on_create: {
                            let db = db.clone();
                            let on_changed = props.on_changed;
                            move |name: String| {
                                let db = db.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    match create_shelf_db(&conn, &name) {
                                        Ok(shelf_id) => {
                                            if let Err(e) = add_book_to_shelf_db(&conn, book_id, shelf_id) {
                                                error!("failed to add book {book_id} to new shelf {shelf_id}: {e}");
                                            }
                                            match list_book_shelves_db(&conn, book_id) {
                                                Ok(shelves) => book_shelf_ids.set(shelves.iter().map(|s| s.id).collect()),
                                                Err(e) => error!("failed to reload shelves for book {book_id}: {e}"),
                                            }
                                        }
                                        Err(e) => error!("failed to create shelf '{name}': {e}"),
                                    }
                                    drop(conn);
                                    on_changed.call(());
                                });
                            }
                        },
                    }
                }

                // Diary Entries
                div {
                    div {
                        class: "mb-2 flex items-center justify-between",
                        label {
                            class: "text-xs font-medium text-gray-500 dark:text-gray-400",
                            "Diary Entries"
                        }
                        button {
                            r#type: "button",
                            onclick: move |_| show_diary_form.set(true),
                            class: "text-xs text-amber-600 hover:text-amber-700 dark:text-amber-400
                                dark:hover:text-amber-300",
                            "+ Add Entry"
                        }
                    }
                    if diary_entries.read().is_empty() {
                        div {
                            class: "rounded-lg border border-dashed border-gray-300 px-4 py-6 text-center dark:border-gray-600",
                            p {
                                class: "text-sm text-gray-500 dark:text-gray-400",
                                "No journal entries yet"
                            }
                            p {
                                class: "mt-1 text-xs text-gray-400 dark:text-gray-500",
                                "Capture your thoughts about this book."
                            }
                        }
                    } else {
                        div {
                            class: "space-y-2 max-h-64 overflow-y-auto",
                            for entry in diary_entries.read().iter() {
                                div {
                                    key: "{entry.id}",
                                    class: "rounded-lg border border-gray-200 bg-gray-50 px-3 py-2
                                        dark:border-gray-700 dark:bg-gray-800/50",
                                    div {
                                        class: "flex items-center justify-between gap-2",
                                        span {
                                            class: "text-xs font-medium text-gray-600 dark:text-gray-400",
                                            "{entry.entry_date}"
                                        }
                                        if let Some(rating) = entry.rating {
                                            div {
                                                class: "flex gap-0.5",
                                                for s in 1..=5 {
                                                    svg {
                                                        class: "h-3 w-3 {diary_star_class(s, rating)}",
                                                        view_box: "0 0 20 20",
                                                        fill: "currentColor",
                                                        path {
                                                            d: "M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Some(ref body) = entry.body {
                                        p {
                                            class: "mt-1 text-sm text-gray-600 dark:text-gray-400 line-clamp-2",
                                            "{body_preview(body)}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Highlights
                div {
                    class: "border-t border-gray-200 pt-4 dark:border-gray-700",
                    label {
                        class: "mb-2 block text-xs font-medium text-gray-500 dark:text-gray-400",
                        "Highlights"
                    }
                    if highlights.read().is_empty() {
                        p {
                            class: "text-sm text-gray-400 dark:text-gray-500 italic",
                            "No highlights yet"
                        }
                    } else {
                        div {
                            class: "space-y-3 max-h-64 overflow-y-auto",
                            for h in highlights.read().iter() {
                                div {
                                    key: "{h.id}",
                                    class: "rounded-lg border border-gray-200 bg-gray-50 px-3 py-2
                                        dark:border-gray-700 dark:bg-gray-800/50",
                                    div {
                                        class: "flex items-start gap-2",
                                        svg {
                                            class: "mt-0.5 h-4 w-4 flex-shrink-0 text-amber-500",
                                            fill: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                d: "M6 17h3l2-4V7H5v6h3zm8 0h3l2-4V7h-6v6h3z",
                                            }
                                        }
                                        if h.text.is_empty() {
                                            p { class: "text-sm text-gray-400 italic", "Bookmark" }
                                        } else {
                                            p {
                                                class: "text-sm text-gray-700 dark:text-gray-300 italic",
                                                "{h.text}"
                                            }
                                        }
                                    }
                                    div {
                                        class: "mt-1 flex items-center gap-2 text-[10px] text-gray-500 dark:text-gray-400",
                                        span {
                                            class: "{clip_type_class(&h.clip_type)} rounded px-1 py-0.5 font-medium uppercase",
                                            "{h.clip_type}"
                                        }
                                        if let Some(loc_start) = h.location_start {
                                            span {
                                                "Loc {loc_start}"
                                                if let Some(loc_end) = h.location_end {
                                                    "-{loc_end}"
                                                }
                                            }
                                        }
                                        if let Some(page) = h.page {
                                            span { "p. {page}" }
                                        }
                                        if let Some(ref clipped_at) = h.clipped_at {
                                            span { "{clipped_at}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Metadata
                if bk.isbn.is_some() || bk.publisher.is_some() || bk.published_date.is_some() || bk.page_count.is_some() {
                    div {
                        class: "space-y-1 border-t border-gray-200 pt-4 dark:border-gray-700",
                        h3 {
                            class: "mb-2 text-xs font-medium text-gray-500 dark:text-gray-400",
                            "Details"
                        }
                        if let Some(ref isbn) = bk.isbn {
                            p {
                                class: "text-xs text-gray-600 dark:text-gray-400",
                                span { class: "font-medium", "ISBN: " }
                                "{isbn}"
                            }
                        }
                        if let Some(ref publisher) = bk.publisher {
                            p {
                                class: "text-xs text-gray-600 dark:text-gray-400",
                                span { class: "font-medium", "Publisher: " }
                                "{publisher}"
                            }
                        }
                        if let Some(ref pub_date) = bk.published_date {
                            p {
                                class: "text-xs text-gray-600 dark:text-gray-400",
                                span { class: "font-medium", "Published: " }
                                "{pub_date}"
                            }
                        }
                        if let Some(pages) = bk.page_count {
                            p {
                                class: "text-xs text-gray-600 dark:text-gray-400",
                                span { class: "font-medium", "Pages: " }
                                "{pages}"
                            }
                        }
                    }
                }

                // Delete
                div {
                    class: "border-t border-gray-200 pt-4 dark:border-gray-700",
                    button {
                        r#type: "button",
                        onclick: {
                            let db = db.clone();
                            let on_deleted = props.on_deleted;
                            let on_close = props.on_close;
                            move |_| {
                                if !*confirm_delete.read() {
                                    confirm_delete.set(true);
                                    return;
                                }
                                let db = db.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    if let Err(e) = delete_book_with_covers_db(&conn, book_id) {
                                        error!("failed to delete book {book_id}: {e}");
                                    }
                                    drop(conn);
                                    on_deleted.call(book_id);
                                    on_close.call(());
                                });
                            }
                        },
                        onmouseleave: move |_| confirm_delete.set(false),
                        class: if *confirm_delete.read() {
                            "rounded-md px-4 py-2 text-sm font-medium transition bg-red-600 text-white hover:bg-red-700"
                        } else {
                            "rounded-md px-4 py-2 text-sm font-medium transition text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
                        },
                        if *confirm_delete.read() { "Confirm Delete" } else { "Delete Book" }
                    }
                }
            }
        }

        // Diary entry form overlay
        if *show_diary_form.read() {
            DiaryEntryForm {
                book_id: book_id,
                book_title: Some(bk.title.clone()),
                on_save: {
                    let db = db.clone();
                    move |_| {
                        let db = db.clone();
                        spawn(async move {
                            let conn = db.conn.lock().unwrap();
                            match list_book_diary_entries_db(&conn, book_id) {
                                Ok(entries) => diary_entries.set(entries),
                                Err(e) => error!("failed to reload diary entries for book {book_id}: {e}"),
                            }
                        });
                    }
                },
                on_close: move |_| show_diary_form.set(false),
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum CoverMode {
    Paste,
    Search,
}

fn clip_type_class(clip_type: &str) -> &'static str {
    match clip_type {
        "highlight" => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400",
        "note" => "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400",
        _ => "bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400",
    }
}

fn diary_star_class(star: i32, rating: i32) -> &'static str {
    if star <= rating {
        "text-amber-400 fill-amber-400"
    } else {
        "text-gray-300 dark:text-gray-600 fill-current"
    }
}

fn body_preview(body: &str) -> String {
    // Try to extract plain text if it's JSON (TipTap format)
    if body.starts_with('{') || body.starts_with('[') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
            if let Some(text) = extract_text_from_tiptap(&val) {
                return text.chars().take(200).collect();
            }
        }
    }
    body.chars().take(200).collect()
}

fn extract_text_from_tiptap(val: &serde_json::Value) -> Option<String> {
    match val {
        serde_json::Value::Object(obj) => {
            if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                return Some(text.to_string());
            }
            if let Some(content) = obj.get("content").and_then(|c| c.as_array()) {
                let texts: Vec<String> = content
                    .iter()
                    .filter_map(extract_text_from_tiptap)
                    .collect();
                if !texts.is_empty() {
                    return Some(texts.join(" "));
                }
            }
            None
        }
        serde_json::Value::Array(arr) => {
            let texts: Vec<String> = arr.iter().filter_map(extract_text_from_tiptap).collect();
            if !texts.is_empty() {
                Some(texts.join(" "))
            } else {
                None
            }
        }
        _ => None,
    }
}
