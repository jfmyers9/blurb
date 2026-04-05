use dioxus::prelude::*;

use crate::data::commands::{list_diary_entries_db, search_highlights_db};
use crate::data::models::Book;
use crate::data::models::{DiaryEntry, HighlightSearchResult};
use crate::DatabaseHandle;

#[derive(Clone, PartialEq)]
struct PaletteCommand {
    id: &'static str,
    label: &'static str,
    keywords: &'static [&'static str],
}

const COMMANDS: &[PaletteCommand] = &[
    PaletteCommand {
        id: "add-book",
        label: "Add Book",
        keywords: &["new", "create", "add"],
    },
    PaletteCommand {
        id: "switch-library",
        label: "Switch to Library",
        keywords: &["view", "books", "library", "home"],
    },
    PaletteCommand {
        id: "switch-diary",
        label: "Switch to Diary",
        keywords: &["view", "journal", "diary", "entries"],
    },
    PaletteCommand {
        id: "toggle-view",
        label: "Toggle Grid/List View",
        keywords: &["grid", "list", "view", "layout", "toggle"],
    },
    PaletteCommand {
        id: "kindle-sync",
        label: "Open Kindle Sync",
        keywords: &["kindle", "sync", "import", "device"],
    },
];

#[derive(Clone, PartialEq)]
enum ResultItem {
    Command {
        id: &'static str,
        label: &'static str,
    },
    BookResult {
        id: i64,
        title: String,
        author: Option<String>,
        cover_url: Option<String>,
    },
    DiaryResult {
        book_id: i64,
        book_title: String,
        entry_date: String,
        body_preview: Option<String>,
    },
    HighlightResult {
        book_id: i64,
        book_title: String,
        book_author: Option<String>,
        text_preview: String,
    },
}

impl ResultItem {
    fn key(&self) -> String {
        match self {
            ResultItem::Command { id, .. } => format!("cmd-{id}"),
            ResultItem::BookResult { id, .. } => format!("book-{id}"),
            ResultItem::DiaryResult {
                book_id,
                entry_date,
                ..
            } => format!("diary-{book_id}-{entry_date}"),
            ResultItem::HighlightResult {
                book_id,
                text_preview,
                ..
            } => {
                format!(
                    "hl-{book_id}-{}",
                    &text_preview[..text_preview.len().min(20)]
                )
            }
        }
    }

    fn target_book_id(&self) -> Option<i64> {
        match self {
            ResultItem::BookResult { id, .. } => Some(*id),
            ResultItem::DiaryResult { book_id, .. }
            | ResultItem::HighlightResult { book_id, .. } => Some(*book_id),
            ResultItem::Command { .. } => None,
        }
    }

    fn command_id(&self) -> Option<&'static str> {
        match self {
            ResultItem::Command { id, .. } => Some(id),
            _ => None,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CommandPaletteProps {
    is_open: bool,
    on_close: EventHandler<()>,
    books: Vec<Book>,
    on_select_book: EventHandler<i64>,
    on_command: EventHandler<String>,
}

#[component]
pub fn CommandPalette(props: CommandPaletteProps) -> Element {
    let db = use_context::<DatabaseHandle>();

    let mut query = use_signal(String::new);
    let mut selected_index = use_signal(|| 0usize);
    let mut diary_entries: Signal<Vec<DiaryEntry>> = use_signal(Vec::new);
    let mut highlights: Signal<Vec<HighlightSearchResult>> = use_signal(Vec::new);

    // Load diary entries when opened
    {
        let db = db.clone();
        let is_open = props.is_open;
        use_effect(move || {
            if is_open {
                let db = db.clone();
                spawn(async move {
                    let conn = db.conn.lock().unwrap();
                    if let Ok(entries) = list_diary_entries_db(&conn) {
                        diary_entries.set(entries);
                    }
                });
            } else {
                query.set(String::new());
                highlights.set(Vec::new());
                selected_index.set(0);
            }
        });
    }

    // Highlight search when query changes
    {
        let db = db.clone();
        use_effect(move || {
            let q = query.read().clone();
            if q.len() >= 3 {
                let db = db.clone();
                spawn(async move {
                    let conn = db.conn.lock().unwrap();
                    match search_highlights_db(&conn, &q) {
                        Ok(results) => highlights.set(results),
                        Err(_) => highlights.set(Vec::new()),
                    }
                });
            } else {
                highlights.set(Vec::new());
            }
        });
    }

    if !props.is_open {
        return rsx! {};
    }

    let lower_q = query.read().to_lowercase();
    let has_query = !lower_q.is_empty();

    // Build all items as a flat Vec with section labels interspersed
    let mut all_items: Vec<ResultItem> = Vec::new();
    let mut section_info: Vec<(&str, usize, usize)> = Vec::new(); // (label, start, count)

    // Commands
    let filtered_commands: Vec<ResultItem> = if has_query {
        COMMANDS
            .iter()
            .filter(|c| {
                c.label.to_lowercase().contains(&lower_q)
                    || c.keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&lower_q))
            })
            .map(|c| ResultItem::Command {
                id: c.id,
                label: c.label,
            })
            .collect()
    } else {
        COMMANDS
            .iter()
            .map(|c| ResultItem::Command {
                id: c.id,
                label: c.label,
            })
            .collect()
    };
    if !filtered_commands.is_empty() {
        section_info.push(("Actions", all_items.len(), filtered_commands.len()));
        all_items.extend(filtered_commands);
    }

    // Books
    if has_query {
        let filtered_books: Vec<ResultItem> = props
            .books
            .iter()
            .filter(|b| {
                b.title.to_lowercase().contains(&lower_q)
                    || b.author
                        .as_ref()
                        .is_some_and(|a| a.to_lowercase().contains(&lower_q))
            })
            .take(10)
            .map(|b| ResultItem::BookResult {
                id: b.id,
                title: b.title.clone(),
                author: b.author.clone(),
                cover_url: b.cover_url.clone(),
            })
            .collect();
        if !filtered_books.is_empty() {
            section_info.push(("Books", all_items.len(), filtered_books.len()));
            all_items.extend(filtered_books);
        }
    }

    // Diary entries
    if has_query {
        let filtered_diary: Vec<ResultItem> = diary_entries
            .read()
            .iter()
            .filter(|e| {
                e.book_title.to_lowercase().contains(&lower_q)
                    || e.body
                        .as_ref()
                        .is_some_and(|b| b.to_lowercase().contains(&lower_q))
            })
            .take(10)
            .map(|e| ResultItem::DiaryResult {
                book_id: e.book_id,
                book_title: e.book_title.clone(),
                entry_date: e.entry_date.clone(),
                body_preview: e.body.as_ref().map(|b| b[..b.len().min(100)].to_string()),
            })
            .collect();
        if !filtered_diary.is_empty() {
            section_info.push(("Diary Entries", all_items.len(), filtered_diary.len()));
            all_items.extend(filtered_diary);
        }
    }

    // Highlights
    {
        let hl_items: Vec<ResultItem> = highlights
            .read()
            .iter()
            .take(10)
            .map(|h| ResultItem::HighlightResult {
                book_id: h.book_id,
                book_title: h.book_title.clone(),
                book_author: h.book_author.clone(),
                text_preview: h.text[..h.text.len().min(120)].to_string(),
            })
            .collect();
        if !hl_items.is_empty() {
            section_info.push(("Highlights", all_items.len(), hl_items.len()));
            all_items.extend(hl_items);
        }
    }

    let total = all_items.len();
    let on_close = props.on_close;
    let on_select_book = props.on_select_book;
    let on_command = props.on_command;

    // Shared execute function using signals to communicate
    let mut execute_signal: Signal<Option<ResultItem>> = use_signal(|| None);

    // Process execute signal
    {
        let pending = execute_signal.read().clone();
        if let Some(item) = pending {
            execute_signal.set(None);
            if let Some(cmd_id) = item.command_id() {
                on_command.call(cmd_id.to_string());
                on_close.call(());
            } else if let Some(book_id) = item.target_book_id() {
                on_select_book.call(book_id);
                on_close.call(());
            }
        }
    }

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-start justify-center bg-black/50 backdrop-blur-sm",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "mt-[20vh] w-full max-w-lg rounded-xl border border-gray-200
                    bg-white shadow-2xl dark:border-gray-700 dark:bg-gray-900",
                onclick: move |e| e.stop_propagation(),

                // Search input
                div {
                    class: "flex items-center border-b border-gray-200 px-4 dark:border-gray-700",
                    svg {
                        class: "mr-2 h-5 w-5 shrink-0 text-gray-400",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
                        }
                    }
                    input {
                        r#type: "text",
                        value: "{query}",
                        oninput: move |e| {
                            query.set(e.value());
                            selected_index.set(0);
                        },
                        onkeydown: {
                            let all_items = all_items.clone();
                            move |e: KeyboardEvent| {
                                match e.key() {
                                    Key::ArrowDown => {
                                        let cur = *selected_index.read();
                                        if cur + 1 < total {
                                            selected_index.set(cur + 1);
                                        }
                                    }
                                    Key::ArrowUp => {
                                        let cur = *selected_index.read();
                                        if cur > 0 {
                                            selected_index.set(cur - 1);
                                        }
                                    }
                                    Key::Enter => {
                                        let cur = *selected_index.read();
                                        if let Some(item) = all_items.get(cur) {
                                            execute_signal.set(Some(item.clone()));
                                        }
                                    }
                                    Key::Escape => {
                                        props.on_close.call(());
                                    }
                                    _ => {}
                                }
                            }
                        },
                        placeholder: "Search books, entries, or actions...",
                        class: "h-12 w-full bg-transparent text-sm text-gray-900 placeholder-gray-400
                            outline-none dark:text-gray-100 dark:placeholder-gray-500",
                        autofocus: true,
                    }
                }

                // Results list
                div {
                    class: "max-h-80 overflow-y-auto p-2",
                    if all_items.is_empty() && has_query {
                        div {
                            class: "flex items-center justify-center py-8 text-sm text-gray-400 dark:text-gray-500",
                            "No results found"
                        }
                    } else if all_items.is_empty() {
                        div {
                            class: "flex items-center justify-center py-8 text-sm text-gray-400 dark:text-gray-500",
                            "Start typing to search..."
                        }
                    } else {
                        for (label, start, count) in section_info.iter() {
                            div {
                                div {
                                    class: "px-2 pb-1 pt-2 text-xs font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500",
                                    "{label}"
                                }
                                for ii in 0..*count {
                                    {
                                        let global_idx = start + ii;
                                        let item = all_items[global_idx].clone();
                                        let cur_selected = *selected_index.read();
                                        let is_active = global_idx == cur_selected;
                                        let active_class = if is_active {
                                            "bg-amber-50 text-amber-900 dark:bg-amber-900/20 dark:text-amber-100"
                                        } else {
                                            "text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800"
                                        };
                                        let item_for_click = item.clone();
                                        rsx! {
                                            button {
                                                key: "{item.key()}",
                                                r#type: "button",
                                                class: "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition {active_class}",
                                                onclick: move |_| {
                                                    execute_signal.set(Some(item_for_click.clone()));
                                                },
                                                onmouseenter: move |_| {
                                                    selected_index.set(global_idx);
                                                },
                                                {render_item_content(&item, &lower_q)}
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

fn render_item_content(item: &ResultItem, query: &str) -> Element {
    match item {
        ResultItem::Command { label, .. } => rsx! {
            span {
                class: "flex h-6 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800",
                ">"
            }
            span { {highlight_text(label, query)} }
        },
        ResultItem::BookResult {
            title,
            author,
            cover_url,
            ..
        } => rsx! {
            if let Some(ref url) = cover_url {
                img {
                    src: "{url}",
                    alt: "",
                    class: "h-8 w-6 shrink-0 rounded object-cover",
                }
            } else {
                span {
                    class: "flex h-8 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800",
                    "B"
                }
            }
            div {
                class: "min-w-0",
                div { class: "truncate font-medium", {highlight_text(title, query)} }
                if let Some(ref a) = author {
                    div { class: "truncate text-xs text-gray-400", {highlight_text(a, query)} }
                }
            }
        },
        ResultItem::DiaryResult {
            book_title,
            entry_date,
            body_preview,
            ..
        } => rsx! {
            span {
                class: "flex h-6 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800",
                "D"
            }
            div {
                class: "min-w-0",
                div {
                    class: "truncate font-medium",
                    {highlight_text(book_title, query)}
                    span { class: "font-normal text-gray-400", " \u{00b7} {entry_date}" }
                }
                if let Some(ref b) = body_preview {
                    div {
                        class: "truncate text-xs text-gray-400",
                        {highlight_text(b, query)}
                    }
                }
            }
        },
        ResultItem::HighlightResult {
            book_title,
            book_author,
            text_preview,
            ..
        } => rsx! {
            span {
                class: "flex h-6 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800",
                "H"
            }
            div {
                class: "min-w-0",
                div {
                    class: "truncate text-xs text-gray-400",
                    {highlight_text(book_title, query)}
                    if let Some(ref a) = book_author {
                        span { " - " {highlight_text(a, query)} }
                    }
                }
                div { class: "truncate", {highlight_text(text_preview, query)} }
            }
        },
    }
}

fn highlight_text(text: &str, query: &str) -> Element {
    if query.is_empty() {
        return rsx! { "{text}" };
    }
    let lower = text.to_lowercase();
    match lower.find(query) {
        None => rsx! { "{text}" },
        Some(i) => {
            let before = &text[..i];
            let matched = &text[i..i + query.len()];
            let after = &text[i + query.len()..];
            rsx! {
                "{before}"
                strong { class: "font-semibold text-amber-600 dark:text-amber-400", "{matched}" }
                "{after}"
            }
        }
    }
}
