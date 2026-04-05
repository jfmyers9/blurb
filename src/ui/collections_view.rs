use dioxus::prelude::*;

use crate::data::commands::{
    add_book_to_collection_db, create_collection_db, delete_collection_db, get_collection_books_db,
    list_books_db, list_collections_db, remove_book_from_collection_db, reorder_collection_db,
};
use crate::data::models::{Book, Collection};
use crate::DatabaseHandle;

#[component]
fn CreateCollectionForm(on_created: EventHandler<Collection>) -> Element {
    let db = use_context::<DatabaseHandle>();
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    rsx! {
        form {
            class: "rounded-xl border border-gray-200 bg-white p-4 shadow-sm
                dark:border-gray-700 dark:bg-gray-800",
            onsubmit: move |e| {
                e.prevent_default();
                let n = name.read().clone();
                let d = description.read().clone();
                let desc = if d.is_empty() { None } else { Some(d) };
                let db = db.clone();
                spawn(async move {
                    let result = {
                        let conn = db.conn.lock().unwrap();
                        create_collection_db(&conn, &n, desc.as_deref())
                    };
                    match result {
                        Ok(col) => {
                            name.set(String::new());
                            description.set(String::new());
                            error.set(None);
                            on_created.call(col);
                        }
                        Err(e) => error.set(Some(e)),
                    }
                });
            },
            div {
                class: "flex flex-col gap-3",
                input {
                    r#type: "text",
                    placeholder: "Collection name",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    class: "rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm
                        text-gray-900 placeholder-gray-400 outline-none
                        focus:border-amber-500 focus:ring-1 focus:ring-amber-500
                        dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100
                        dark:placeholder-gray-500",
                }
                input {
                    r#type: "text",
                    placeholder: "Description (optional)",
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                    class: "rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm
                        text-gray-900 placeholder-gray-400 outline-none
                        focus:border-amber-500 focus:ring-1 focus:ring-amber-500
                        dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100
                        dark:placeholder-gray-500",
                }
                div {
                    class: "flex items-center gap-2",
                    button {
                        r#type: "submit",
                        class: "rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white
                            shadow-sm transition hover:bg-amber-700 active:scale-95",
                        "Create Collection"
                    }
                    if let Some(err) = error.read().as_ref() {
                        span {
                            class: "text-sm text-red-500",
                            "{err}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BookPicker(
    collection_id: i64,
    existing_book_ids: Vec<i64>,
    on_added: EventHandler<()>,
) -> Element {
    let db = use_context::<DatabaseHandle>();
    let mut search = use_signal(String::new);
    let mut all_books: Signal<Vec<Book>> = use_signal(Vec::new);
    let mut loaded = use_signal(|| false);

    if !*loaded.read() {
        let db = db.clone();
        spawn(async move {
            let conn = db.conn.lock().unwrap();
            if let Ok(books) = list_books_db(&conn) {
                all_books.set(books);
            }
            loaded.set(true);
        });
    }

    let query = search.read().to_lowercase();
    let available: Vec<Book> = all_books
        .read()
        .iter()
        .filter(|b| !existing_book_ids.contains(&b.id))
        .filter(|b| {
            if query.is_empty() {
                return true;
            }
            b.title.to_lowercase().contains(&query)
                || b.author
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&query)
        })
        .take(10)
        .cloned()
        .collect();

    rsx! {
        div {
            class: "mt-3 rounded-lg border border-gray-200 bg-gray-50 p-3
                dark:border-gray-600 dark:bg-gray-700",
            input {
                r#type: "text",
                placeholder: "Search books to add...",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
                class: "mb-2 w-full rounded-md border border-gray-200 bg-white px-3 py-1.5
                    text-sm text-gray-900 placeholder-gray-400 outline-none
                    focus:border-amber-500 dark:border-gray-600 dark:bg-gray-800
                    dark:text-gray-100 dark:placeholder-gray-500",
            }
            if available.is_empty() {
                p {
                    class: "text-xs text-gray-400 dark:text-gray-500",
                    "No books available to add"
                }
            }
            for book in available.iter() {
                {
                    let bid = book.id;
                    let cid = collection_id;
                    let title = book.title.clone();
                    let author = book.author.clone().unwrap_or_default();
                    let db = db.clone();
                    rsx! {
                        button {
                            r#type: "button",
                            key: "{bid}",
                            onclick: move |_| {
                                let db = db.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    let _ = add_book_to_collection_db(&conn, cid, bid);
                                    drop(conn);
                                    on_added.call(());
                                });
                            },
                            class: "flex w-full items-center gap-2 rounded-md px-2 py-1.5
                                text-left text-sm transition hover:bg-amber-50
                                dark:hover:bg-amber-900/20",
                            svg {
                                class: "h-4 w-4 shrink-0 text-amber-500",
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
                            span {
                                class: "text-gray-800 dark:text-gray-200",
                                "{title}"
                            }
                            if !author.is_empty() {
                                span {
                                    class: "text-gray-400 dark:text-gray-500",
                                    "by {author}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CollectionCard(
    collection: Collection,
    on_changed: EventHandler<()>,
    on_select_book: EventHandler<i64>,
) -> Element {
    let db = use_context::<DatabaseHandle>();
    let mut expanded = use_signal(|| false);
    let mut books: Signal<Vec<Book>> = use_signal(Vec::new);
    let mut show_picker = use_signal(|| false);
    let cid = collection.id;

    let count_label = if collection.book_count == 1 {
        "1 book".to_string()
    } else {
        format!("{} books", collection.book_count)
    };

    let db_expand = db.clone();
    let db_delete = db.clone();
    let db_picker = db.clone();
    let db_books = db.clone();

    rsx! {
        div {
            class: "rounded-xl border border-gray-200 bg-white shadow-sm transition
                dark:border-gray-700 dark:bg-gray-800",
            div {
                class: "flex cursor-pointer items-center justify-between p-4",
                onclick: move |_| {
                    let is_expanding = !*expanded.read();
                    expanded.set(is_expanding);
                    if is_expanding {
                        let db = db_expand.clone();
                        spawn(async move {
                            let conn = db.conn.lock().unwrap();
                            if let Ok(b) = get_collection_books_db(&conn, cid) {
                                books.set(b);
                            }
                        });
                    }
                },
                div {
                    class: "flex-1",
                    h3 {
                        class: "text-base font-semibold text-gray-900 dark:text-gray-100",
                        "{collection.name}"
                    }
                    if let Some(ref desc) = collection.description {
                        p {
                            class: "mt-0.5 text-sm text-gray-500 dark:text-gray-400",
                            "{desc}"
                        }
                    }
                    span {
                        class: "mt-1 inline-block text-xs text-gray-400 dark:text-gray-500",
                        "{count_label}"
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    button {
                        r#type: "button",
                        title: "Delete collection",
                        onclick: move |e| {
                            e.stop_propagation();
                            let db = db_delete.clone();
                            spawn(async move {
                                let conn = db.conn.lock().unwrap();
                                let _ = delete_collection_db(&conn, cid);
                                drop(conn);
                                on_changed.call(());
                            });
                        },
                        class: "rounded-md p-1 text-gray-300 transition hover:text-red-500
                            dark:text-gray-600 dark:hover:text-red-400",
                        svg {
                            class: "h-4 w-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                            }
                        }
                    }
                    svg {
                        class: if *expanded.read() {
                            "h-5 w-5 rotate-180 text-gray-400 transition-transform"
                        } else {
                            "h-5 w-5 text-gray-400 transition-transform"
                        },
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M19 9l-7 7-7-7",
                        }
                    }
                }
            }

            if *expanded.read() {
                div {
                    class: "border-t border-gray-100 px-4 pb-4 pt-3 dark:border-gray-700",
                    if books.read().is_empty() {
                        p {
                            class: "text-sm text-gray-400 dark:text-gray-500",
                            "No books in this collection yet."
                        }
                    }
                    {
                        let books_snapshot: Vec<(usize, i64, String, String, Option<String>)> = {
                            let br = books.read();
                            br.iter().enumerate().map(|(idx, b)| {
                                (idx, b.id, b.title.clone(), b.author.clone().unwrap_or_default(), b.cover_url.clone())
                            }).collect()
                        };
                        let book_count = books_snapshot.len();
                        rsx! {
                            for (idx, bid, title, author, cover) in books_snapshot {
                                {
                                    let can_move_up = idx > 0;
                                    let can_move_down = idx + 1 < book_count;
                                    let db_up = db_books.clone();
                                    let db_down = db_books.clone();
                                    let db_rm = db_books.clone();
                                    rsx! {
                                        div {
                                            key: "{bid}",
                                            class: "group flex items-center gap-3 rounded-lg px-2 py-2
                                                transition hover:bg-gray-50 dark:hover:bg-gray-700/50",
                                            span {
                                                class: "w-6 shrink-0 text-center text-xs font-medium
                                                    text-gray-400 dark:text-gray-500",
                                                "{idx + 1}"
                                            }
                                            if let Some(ref url) = cover {
                                                img {
                                                    src: "{url}",
                                                    class: "h-10 w-7 shrink-0 rounded object-cover shadow-sm",
                                                }
                                            } else {
                                                div {
                                                    class: "flex h-10 w-7 shrink-0 items-center justify-center
                                                        rounded bg-gray-200 dark:bg-gray-600",
                                                    span {
                                                        class: "text-xs text-gray-400",
                                                        "?"
                                                    }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                onclick: move |_| on_select_book.call(bid),
                                                class: "flex-1 text-left",
                                                span {
                                                    class: "text-sm font-medium text-gray-800 dark:text-gray-200",
                                                    "{title}"
                                                }
                                                if !author.is_empty() {
                                                    span {
                                                        class: "ml-2 text-xs text-gray-400 dark:text-gray-500",
                                                        "by {author}"
                                                    }
                                                }
                                            }
                                            div {
                                                class: "flex items-center gap-1 opacity-0 transition
                                                    group-hover:opacity-100",
                                                if can_move_up {
                                                    button {
                                                        r#type: "button",
                                                        title: "Move up",
                                                        onclick: move |_| {
                                                            let mut ids: Vec<i64> = books.read().iter().map(|b| b.id).collect();
                                                            ids.swap(idx, idx - 1);
                                                            let db = db_up.clone();
                                                            spawn(async move {
                                                                let conn = db.conn.lock().unwrap();
                                                                let _ = reorder_collection_db(&conn, cid, &ids);
                                                                if let Ok(b) = get_collection_books_db(&conn, cid) {
                                                                    books.set(b);
                                                                }
                                                            });
                                                        },
                                                        class: "rounded p-0.5 text-gray-400
                                                            hover:text-amber-600 dark:hover:text-amber-400",
                                                        svg {
                                                            class: "h-4 w-4",
                                                            fill: "none",
                                                            stroke: "currentColor",
                                                            view_box: "0 0 24 24",
                                                            path {
                                                                stroke_linecap: "round",
                                                                stroke_linejoin: "round",
                                                                stroke_width: "2",
                                                                d: "M5 15l7-7 7 7",
                                                            }
                                                        }
                                                    }
                                                }
                                                if can_move_down {
                                                    button {
                                                        r#type: "button",
                                                        title: "Move down",
                                                        onclick: move |_| {
                                                            let mut ids: Vec<i64> = books.read().iter().map(|b| b.id).collect();
                                                            ids.swap(idx, idx + 1);
                                                            let db = db_down.clone();
                                                            spawn(async move {
                                                                let conn = db.conn.lock().unwrap();
                                                                let _ = reorder_collection_db(&conn, cid, &ids);
                                                                if let Ok(b) = get_collection_books_db(&conn, cid) {
                                                                    books.set(b);
                                                                }
                                                            });
                                                        },
                                                        class: "rounded p-0.5 text-gray-400
                                                            hover:text-amber-600 dark:hover:text-amber-400",
                                                        svg {
                                                            class: "h-4 w-4",
                                                            fill: "none",
                                                            stroke: "currentColor",
                                                            view_box: "0 0 24 24",
                                                            path {
                                                                stroke_linecap: "round",
                                                                stroke_linejoin: "round",
                                                                stroke_width: "2",
                                                                d: "M19 9l-7 7-7-7",
                                                            }
                                                        }
                                                    }
                                                }
                                                button {
                                                    r#type: "button",
                                                    title: "Remove from collection",
                                                    onclick: move |_| {
                                                        let db = db_rm.clone();
                                                        spawn(async move {
                                                            let conn = db.conn.lock().unwrap();
                                                            let _ = remove_book_from_collection_db(&conn, cid, bid);
                                                            if let Ok(b) = get_collection_books_db(&conn, cid) {
                                                                books.set(b);
                                                            }
                                                            drop(conn);
                                                            on_changed.call(());
                                                        });
                                                    },
                                                    class: "rounded p-0.5 text-gray-400
                                                        hover:text-red-500 dark:hover:text-red-400",
                                                    svg {
                                                        class: "h-4 w-4",
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
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        class: "mt-3",
                        if *show_picker.read() {
                            BookPicker {
                                collection_id: cid,
                                existing_book_ids: books.read().iter().map(|b| b.id).collect(),
                                on_added: move |_| {
                                    let db = db_picker.clone();
                                    spawn(async move {
                                        let conn = db.conn.lock().unwrap();
                                        if let Ok(b) = get_collection_books_db(&conn, cid) {
                                            books.set(b);
                                        }
                                        drop(conn);
                                        on_changed.call(());
                                    });
                                },
                            }
                        }
                        button {
                            r#type: "button",
                            onclick: move |_| {
                                let current = *show_picker.read();
                                show_picker.set(!current);
                            },
                            class: "mt-2 text-sm font-medium text-amber-600 transition
                                hover:text-amber-700 dark:text-amber-400 dark:hover:text-amber-300",
                            if *show_picker.read() { "Done adding" } else { "+ Add books" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CollectionsView(on_select_book: EventHandler<i64>) -> Element {
    let db = use_context::<DatabaseHandle>();
    let mut collections: Signal<Vec<Collection>> = use_signal(Vec::new);
    let mut show_create = use_signal(|| false);

    let reload = move || {
        let db = db.clone();
        spawn(async move {
            let conn = db.conn.lock().unwrap();
            if let Ok(c) = list_collections_db(&conn) {
                collections.set(c);
            }
        });
    };

    {
        let reload = reload.clone();
        use_effect(move || {
            reload();
        });
    }

    rsx! {
        div {
            class: "mx-auto max-w-3xl px-6 py-6",
            div {
                class: "mb-6 flex items-center justify-between",
                h2 {
                    class: "text-lg font-bold text-gray-900 dark:text-gray-100",
                    "Collections"
                }
                button {
                    r#type: "button",
                    onclick: move |_| {
                        let current = *show_create.read();
                        show_create.set(!current);
                    },
                    class: "rounded-lg bg-amber-600 px-3 py-1.5 text-sm font-medium text-white
                        shadow-sm transition hover:bg-amber-700 active:scale-95",
                    if *show_create.read() { "Cancel" } else { "New Collection" }
                }
            }

            if *show_create.read() {
                div {
                    class: "mb-6",
                    CreateCollectionForm {
                        on_created: {
                            let reload = reload.clone();
                            move |_col: Collection| {
                                show_create.set(false);
                                reload();
                            }
                        },
                    }
                }
            }

            if collections.read().is_empty() {
                div {
                    class: "flex flex-col items-center justify-center py-16 text-center",
                    div {
                        class: "mb-3 text-5xl opacity-30",
                        "\u{1f4cb}"
                    }
                    p {
                        class: "text-sm text-gray-500 dark:text-gray-400",
                        "Create your first collection to curate ordered book lists."
                    }
                }
            }

            div {
                class: "flex flex-col gap-4",
                for col in collections.read().iter() {
                    CollectionCard {
                        key: "{col.id}",
                        collection: col.clone(),
                        on_changed: {
                            let reload = reload.clone();
                            move |_| reload()
                        },
                        on_select_book: move |id: i64| on_select_book.call(id),
                    }
                }
            }
        }
    }
}
