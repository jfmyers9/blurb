use dioxus::prelude::*;

use crate::data::commands::add_book_db;
use crate::services::metadata::{self, BookMetadata};
use crate::DatabaseHandle;

#[derive(Props, Clone, PartialEq)]
pub struct AddBookFormProps {
    on_close: EventHandler<()>,
    on_added: EventHandler<i64>,
}

#[component]
pub fn AddBookForm(props: AddBookFormProps) -> Element {
    let db = use_context::<DatabaseHandle>();

    let mut title_val = use_signal(String::new);
    let mut author_val = use_signal(String::new);
    let mut isbn_val = use_signal(String::new);
    let mut cover_url = use_signal(String::new);
    let mut selected_meta = use_signal(|| None::<BookMetadata>);
    let mut submitting = use_signal(|| false);

    // ISBN lookup
    let mut looking_up = use_signal(|| false);
    let mut lookup_error = use_signal(|| None::<String>);

    // Book search
    let mut search_query = use_signal(String::new);
    let mut search_results = use_signal(Vec::<BookMetadata>::new);
    let mut searching = use_signal(|| false);
    let mut search_error = use_signal(|| None::<String>);
    let mut search_done = use_signal(|| false);

    // Cover search
    let mut cover_results = use_signal(Vec::<BookMetadata>::new);
    let mut searching_covers = use_signal(|| false);

    let handle_search = move || {
        let query = search_query.read().trim().to_string();
        if query.is_empty() || *searching.read() {
            return;
        }
        searching.set(true);
        search_results.set(vec![]);
        search_error.set(None);
        search_done.set(false);
        spawn(async move {
            match metadata::search_covers(&query).await {
                Ok(results) => {
                    search_results.set(results);
                    search_done.set(true);
                }
                Err(e) => search_error.set(Some(e)),
            }
            searching.set(false);
        });
    };

    let handle_lookup = {
        move || {
            let isbn = isbn_val.read().trim().to_string();
            if isbn.is_empty() {
                return;
            }
            looking_up.set(true);
            lookup_error.set(None);
            spawn(async move {
                match metadata::lookup(&isbn).await {
                    Ok(meta) => {
                        if let Some(ref t) = meta.title {
                            if title_val.read().trim().is_empty() {
                                title_val.set(t.clone());
                            }
                        }
                        if let Some(ref a) = meta.author {
                            if author_val.read().trim().is_empty() {
                                author_val.set(a.clone());
                            }
                        }
                        selected_meta.set(Some(meta));
                    }
                    Err(e) => lookup_error.set(Some(e)),
                }
                looking_up.set(false);
            });
        }
    };

    let handle_submit = {
        let db = db.clone();
        let on_close = props.on_close;
        let on_added = props.on_added;
        move || {
            let t = title_val.read().trim().to_string();
            if t.is_empty() {
                return;
            }
            submitting.set(true);
            let a = author_val.read().trim().to_string();
            let isbn = isbn_val.read().trim().to_string();
            let cover = cover_url.read().trim().to_string();
            let meta = selected_meta.read().clone();
            let db = db.clone();
            spawn(async move {
                let final_cover = if cover.is_empty() {
                    meta.as_ref().and_then(|m| m.cover_url.clone())
                } else {
                    Some(cover)
                };
                let conn = db.conn.lock().unwrap();
                let result = add_book_db(
                    &conn,
                    &t,
                    if a.is_empty() { None } else { Some(a.as_str()) },
                    if isbn.is_empty() {
                        None
                    } else {
                        Some(isbn.as_str())
                    },
                    None, // asin
                    final_cover.as_deref(),
                    meta.as_ref().and_then(|m| m.description.as_deref()),
                    meta.as_ref().and_then(|m| m.publisher.as_deref()),
                    meta.as_ref().and_then(|m| m.published_date.as_deref()),
                    meta.as_ref().and_then(|m| m.page_count),
                );
                drop(conn);
                submitting.set(false);
                if let Ok(new_id) = result {
                    on_added.call(new_id);
                    on_close.call(());
                }
            });
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",

            // Backdrop
            div {
                class: "absolute inset-0 bg-black/50",
                onclick: move |_| props.on_close.call(()),
            }

            // Modal
            div {
                class: "relative z-10 w-full max-w-md rounded-xl bg-white p-6
                    shadow-xl dark:bg-gray-800 max-h-[90vh] overflow-y-auto",

                h2 {
                    class: "mb-4 text-lg font-semibold text-gray-900 dark:text-gray-100",
                    "Add a Book"
                }

                div {
                    class: "space-y-3",

                    // Book search
                    div {
                        label {
                            class: "mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Search books"
                        }
                        div {
                            class: "flex gap-2",
                            input {
                                r#type: "text",
                                value: "{search_query}",
                                oninput: move |evt: Event<FormData>| {
                                    let val = evt.value();
                                    search_query.set(val.clone());
                                    if val.trim().is_empty() {
                                        search_results.set(vec![]);
                                        search_done.set(false);
                                        search_error.set(None);
                                    }
                                },
                                onkeydown: {
                                    let mut handle_search = handle_search;
                                    move |evt: Event<KeyboardData>| {
                                        if evt.key() == Key::Enter {
                                            handle_search();
                                        }
                                    }
                                },
                                placeholder: "Search by title or author...",
                                class: "flex-1 rounded-md border border-gray-300 bg-white px-3
                                    py-2 text-sm text-gray-900 dark:border-gray-600
                                    dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                                    focus:ring-amber-500 focus:outline-none",
                            }
                            button {
                                r#type: "button",
                                onclick: {
                                    let mut handle_search = handle_search;
                                    move |_| handle_search()
                                },
                                disabled: *searching.read() || search_query.read().trim().is_empty(),
                                class: "rounded-md bg-gray-100 px-3 py-2 text-sm font-medium
                                    text-gray-700 hover:bg-gray-200 disabled:opacity-50
                                    disabled:cursor-not-allowed dark:bg-gray-600
                                    dark:text-gray-200 dark:hover:bg-gray-500",
                                if *searching.read() { "Searching..." } else { "Search" }
                            }
                        }
                        if !search_results.read().is_empty() {
                            ul {
                                class: "mt-2 max-h-52 overflow-y-auto rounded-md border
                                    border-gray-200 dark:border-gray-600",
                                for (i, result) in search_results.read().iter().enumerate() {
                                    {
                                        let result = result.clone();
                                        rsx! {
                                            li {
                                                key: "{i}",
                                                button {
                                                    r#type: "button",
                                                    onclick: {
                                                        let result = result.clone();
                                                        move |_| {
                                                            if let Some(ref t) = result.title {
                                                                title_val.set(t.clone());
                                                            }
                                                            if let Some(ref a) = result.author {
                                                                author_val.set(a.clone());
                                                            }
                                                            if let Some(ref isbn) = result.isbn {
                                                                isbn_val.set(isbn.clone());
                                                            }
                                                            if let Some(ref url) = result.cover_url {
                                                                cover_url.set(url.clone());
                                                            }
                                                            selected_meta.set(Some(result.clone()));
                                                            search_results.set(vec![]);
                                                            search_done.set(false);
                                                        }
                                                    },
                                                    class: "flex w-full items-center gap-3 px-3 py-2 text-left
                                                        text-sm hover:bg-gray-100 dark:hover:bg-gray-700
                                                        text-gray-900 dark:text-gray-100",
                                                    if let Some(ref url) = result.cover_url {
                                                        img {
                                                            src: "{url}",
                                                            alt: "",
                                                            class: "h-10 w-7 flex-shrink-0 rounded object-cover",
                                                        }
                                                    } else {
                                                        div {
                                                            class: "h-10 w-7 flex-shrink-0 rounded bg-gray-200
                                                                dark:bg-gray-600",
                                                        }
                                                    }
                                                    div {
                                                        class: "min-w-0",
                                                        p {
                                                            class: "truncate font-medium",
                                                            "{result.title.as_deref().unwrap_or(\"Untitled\")}"
                                                        }
                                                        if let Some(ref a) = result.author {
                                                            p {
                                                                class: "truncate text-xs text-gray-500 dark:text-gray-400",
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
                        if let Some(ref err) = *search_error.read() {
                            p { class: "mt-1 text-xs text-red-500", "{err}" }
                        }
                        if *search_done.read() && search_results.read().is_empty() && search_error.read().is_none() {
                            p { class: "mt-1 text-xs text-gray-500 dark:text-gray-400", "No results found" }
                        }
                    }

                    // ISBN
                    div {
                        label {
                            class: "mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300",
                            "ISBN"
                        }
                        div {
                            class: "flex gap-2",
                            input {
                                r#type: "text",
                                value: "{isbn_val}",
                                oninput: move |evt: Event<FormData>| isbn_val.set(evt.value()),
                                placeholder: "978-0-14-143951-8",
                                class: "flex-1 rounded-md border border-gray-300 bg-white px-3
                                    py-2 text-sm text-gray-900 dark:border-gray-600
                                    dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                                    focus:ring-amber-500 focus:outline-none",
                            }
                            button {
                                r#type: "button",
                                onclick: {
                                    let mut handle_lookup = handle_lookup;
                                    move |_| handle_lookup()
                                },
                                disabled: *looking_up.read() || isbn_val.read().trim().is_empty(),
                                class: "rounded-md bg-gray-100 px-3 py-2 text-sm font-medium
                                    text-gray-700 hover:bg-gray-200 disabled:opacity-50
                                    disabled:cursor-not-allowed dark:bg-gray-600
                                    dark:text-gray-200 dark:hover:bg-gray-500",
                                if *looking_up.read() { "Looking up..." } else { "Look up" }
                            }
                        }
                        if let Some(ref err) = *lookup_error.read() {
                            p { class: "mt-1 text-xs text-red-500", "{err}" }
                        }
                    }

                    // Cover URL
                    div {
                        label {
                            class: "mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Cover URL"
                        }
                        input {
                            r#type: "url",
                            value: "{cover_url}",
                            oninput: move |evt: Event<FormData>| cover_url.set(evt.value()),
                            placeholder: "https://example.com/cover.jpg",
                            class: "w-full rounded-md border border-gray-300 bg-white px-3
                                py-2 text-sm text-gray-900 dark:border-gray-600
                                dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                                focus:ring-amber-500 focus:outline-none",
                        }
                    }

                    // Cover preview
                    {
                        let preview_url = if !cover_url.read().trim().is_empty() {
                            Some(cover_url.read().trim().to_string())
                        } else {
                            selected_meta.read().as_ref().and_then(|m| m.cover_url.clone())
                        };
                        if let Some(url) = preview_url {
                            rsx! {
                                div {
                                    class: "flex justify-center",
                                    img {
                                        src: "{url}",
                                        alt: "Cover preview",
                                        class: "h-32 rounded-md shadow-sm object-contain",
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }

                    // Find cover button
                    if !title_val.read().trim().is_empty() {
                        div {
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    let query = format!("{} {}", title_val.read(), author_val.read()).trim().to_string();
                                    searching_covers.set(true);
                                    cover_results.set(vec![]);
                                    spawn(async move {
                                        if let Ok(results) = metadata::search_covers(&query).await {
                                            cover_results.set(results);
                                        }
                                        searching_covers.set(false);
                                    });
                                },
                                disabled: *searching_covers.read(),
                                class: "rounded-md bg-gray-100 px-3 py-2 text-sm font-medium
                                    text-gray-700 hover:bg-gray-200 disabled:opacity-50
                                    disabled:cursor-not-allowed dark:bg-gray-600
                                    dark:text-gray-200 dark:hover:bg-gray-500",
                                if *searching_covers.read() { "Searching..." } else { "Find cover" }
                            }
                            if !cover_results.read().is_empty() {
                                div {
                                    class: "mt-2 grid grid-cols-4 gap-2",
                                    for (i, result) in cover_results.read().iter().enumerate() {
                                        if let Some(ref url) = result.cover_url {
                                            {
                                                let url = url.clone();
                                                let current_cover = cover_url.read().clone();
                                                let is_selected = current_cover == url;
                                                rsx! {
                                                    button {
                                                        key: "{i}",
                                                        r#type: "button",
                                                        onclick: {
                                                            let url = url.clone();
                                                            move |_| {
                                                                cover_url.set(url.clone());
                                                                cover_results.set(vec![]);
                                                            }
                                                        },
                                                        class: if is_selected {
                                                            "overflow-hidden rounded-md border-2 p-0.5 hover:border-amber-500 transition-colors border-amber-500"
                                                        } else {
                                                            "overflow-hidden rounded-md border-2 p-0.5 hover:border-amber-500 transition-colors border-gray-200 dark:border-gray-600"
                                                        },
                                                        img {
                                                            src: "{url}",
                                                            alt: "{result.title.as_deref().unwrap_or(\"Cover option\")}",
                                                            class: "h-20 w-full object-contain",
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

                    // Title
                    div {
                        label {
                            class: "mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Title "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            value: "{title_val}",
                            oninput: move |evt: Event<FormData>| title_val.set(evt.value()),
                            autofocus: true,
                            class: "w-full rounded-md border border-gray-300 bg-white px-3
                                py-2 text-sm text-gray-900 dark:border-gray-600
                                dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                                focus:ring-amber-500 focus:outline-none",
                        }
                    }

                    // Author
                    div {
                        label {
                            class: "mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300",
                            "Author"
                        }
                        input {
                            r#type: "text",
                            value: "{author_val}",
                            oninput: move |evt: Event<FormData>| author_val.set(evt.value()),
                            class: "w-full rounded-md border border-gray-300 bg-white px-3
                                py-2 text-sm text-gray-900 dark:border-gray-600
                                dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                                focus:ring-amber-500 focus:outline-none",
                        }
                    }

                    // Metadata preview
                    {
                        let meta = selected_meta.read();
                        if let Some(ref m) = *meta {
                            if m.publisher.is_some() || m.page_count.is_some() {
                                rsx! {
                                    div {
                                        class: "rounded-md bg-gray-50 p-3 text-xs text-gray-600
                                            dark:bg-gray-700 dark:text-gray-300 space-y-0.5",
                                        if let Some(ref p) = m.publisher {
                                            p { "Publisher: {p}" }
                                        }
                                        if let Some(ref d) = m.published_date {
                                            p { "Published: {d}" }
                                        }
                                        if let Some(pages) = m.page_count {
                                            p { "Pages: {pages}" }
                                        }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }

                // Actions
                div {
                    class: "mt-6 flex justify-end gap-3",
                    button {
                        r#type: "button",
                        onclick: move |_| props.on_close.call(()),
                        class: "rounded-md px-4 py-2 text-sm font-medium text-gray-600
                            hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200",
                        "Cancel"
                    }
                    button {
                        r#type: "button",
                        disabled: *submitting.read() || title_val.read().trim().is_empty(),
                        onclick: {
                            let mut handle_submit = handle_submit.clone();
                            move |_| handle_submit()
                        },
                        class: "rounded-md bg-amber-600 px-4 py-2 text-sm font-medium
                            text-white hover:bg-amber-700 disabled:opacity-50
                            disabled:cursor-not-allowed",
                        if *submitting.read() { "Adding..." } else { "Add Book" }
                    }
                }
            }
        }
    }
}
