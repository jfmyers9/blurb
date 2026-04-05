use std::collections::HashMap;

use dioxus::prelude::*;

use crate::data::commands::{get_all_settings_db, set_setting_db};
use crate::DatabaseHandle;

const SECTION_HEADING: &str =
    "text-xs font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-3";
const CARD: &str = "rounded-xl border border-gray-200 bg-white p-5 shadow-sm \
    dark:border-gray-800 dark:bg-gray-900";
const LABEL: &str = "text-sm font-medium text-gray-700 dark:text-gray-300";
const SELECT: &str = "rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-sm \
    text-gray-700 shadow-sm focus:border-amber-500 focus:outline-none focus:ring-1 \
    focus:ring-amber-500 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200";
const STAT_VALUE: &str = "text-2xl font-bold text-gray-900 dark:text-gray-100";
const STAT_LABEL: &str = "text-xs text-gray-500 dark:text-gray-400";

#[component]
pub fn SettingsView(on_close: EventHandler<()>) -> Element {
    let db = use_context::<DatabaseHandle>();

    let mut settings: Signal<HashMap<String, String>> = use_signal(HashMap::new);
    let mut db_path: Signal<String> = use_signal(String::new);
    let mut db_size: Signal<String> = use_signal(String::new);
    let mut book_count: Signal<usize> = use_signal(|| 0);
    let mut diary_count: Signal<usize> = use_signal(|| 0);
    let mut highlight_count: Signal<usize> = use_signal(|| 0);

    {
        let db = db.clone();
        use_effect(move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                if let Ok(s) = get_all_settings_db(&conn) {
                    settings.set(s);
                }

                let path = dirs::data_dir()
                    .map(|d| d.join("com.blurb.app/books.db"))
                    .unwrap_or_default();
                db_path.set(path.to_string_lossy().to_string());
                if let Ok(meta) = std::fs::metadata(&path) {
                    let bytes = meta.len();
                    if bytes < 1024 {
                        db_size.set(format!("{bytes} B"));
                    } else if bytes < 1024 * 1024 {
                        db_size.set(format!("{:.1} KB", bytes as f64 / 1024.0));
                    } else {
                        db_size.set(format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)));
                    }
                }

                if let Ok(count) =
                    conn.query_row("SELECT COUNT(*) FROM books", [], |r| r.get::<_, usize>(0))
                {
                    book_count.set(count);
                }
                if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM diary_entries", [], |r| {
                    r.get::<_, usize>(0)
                }) {
                    diary_count.set(count);
                }
                if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM highlights", [], |r| {
                    r.get::<_, usize>(0)
                }) {
                    highlight_count.set(count);
                }
            });
        });
    }

    let save_setting = {
        let db = db.clone();
        move |key: String, value: String| {
            let db = db.clone();
            let k = key.clone();
            let v = value.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                let _ = set_setting_db(&conn, &k, &v);
            });
            let mut s = settings.write();
            s.insert(key, value);
        }
    };

    let theme = settings
        .read()
        .get("theme")
        .cloned()
        .unwrap_or_else(|| "system".to_string());
    let default_view = settings
        .read()
        .get("default_view_mode")
        .cloned()
        .unwrap_or_else(|| "grid".to_string());
    let default_sort = settings
        .read()
        .get("default_sort")
        .cloned()
        .unwrap_or_else(|| "date_added".to_string());
    let default_status = settings
        .read()
        .get("default_status")
        .cloned()
        .unwrap_or_else(|| "want_to_read".to_string());

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-black/40 backdrop-blur-sm",
            onclick: move |_| on_close.call(()),

            div {
                class: "my-8 w-full max-w-2xl rounded-2xl border border-gray-200 bg-gray-50 p-8 shadow-2xl dark:border-gray-700 dark:bg-gray-950",
                onclick: move |e: MouseEvent| e.stop_propagation(),

                // Header
                div {
                    class: "mb-8 flex items-center justify-between",
                    h2 {
                        class: "text-2xl font-bold text-gray-900 dark:text-gray-100",
                        "Settings"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| on_close.call(()),
                        class: "flex h-8 w-8 items-center justify-center rounded-full text-gray-400 transition hover:bg-gray-200 hover:text-gray-600 dark:hover:bg-gray-800 dark:hover:text-gray-300",
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
                    class: "space-y-8",

                    // Appearance
                    div {
                        h3 { class: SECTION_HEADING, "Appearance" }
                        div {
                            class: CARD,
                            div {
                                class: "flex items-center justify-between",
                                label { class: LABEL, "Theme" }
                                select {
                                    class: SELECT,
                                    value: "{theme}",
                                    onchange: {
                                        let mut save_setting = save_setting.clone();
                                        move |e: FormEvent| {
                                            save_setting("theme".to_string(), e.value());
                                        }
                                    },
                                    option { value: "system", "System" }
                                    option { value: "light", "Light" }
                                    option { value: "dark", "Dark" }
                                }
                            }
                        }
                    }

                    // Library Defaults
                    div {
                        h3 { class: SECTION_HEADING, "Library Defaults" }
                        div {
                            class: "{CARD} space-y-4",
                            div {
                                class: "flex items-center justify-between",
                                label { class: LABEL, "Default view" }
                                select {
                                    class: SELECT,
                                    value: "{default_view}",
                                    onchange: {
                                        let mut save_setting = save_setting.clone();
                                        move |e: FormEvent| {
                                            save_setting("default_view_mode".to_string(), e.value());
                                        }
                                    },
                                    option { value: "grid", "Grid" }
                                    option { value: "list", "List" }
                                }
                            }
                            div {
                                class: "flex items-center justify-between",
                                label { class: LABEL, "Default sort" }
                                select {
                                    class: SELECT,
                                    value: "{default_sort}",
                                    onchange: {
                                        let mut save_setting = save_setting.clone();
                                        move |e: FormEvent| {
                                            save_setting("default_sort".to_string(), e.value());
                                        }
                                    },
                                    option { value: "date_added", "Date Added" }
                                    option { value: "title", "Title" }
                                    option { value: "author", "Author" }
                                    option { value: "rating", "Rating" }
                                }
                            }
                        }
                    }

                    // Reading
                    div {
                        h3 { class: SECTION_HEADING, "Reading" }
                        div {
                            class: CARD,
                            div {
                                class: "flex items-center justify-between",
                                label { class: LABEL, "Default status for new books" }
                                select {
                                    class: SELECT,
                                    value: "{default_status}",
                                    onchange: {
                                        let mut save_setting = save_setting.clone();
                                        move |e: FormEvent| {
                                            save_setting("default_status".to_string(), e.value());
                                        }
                                    },
                                    option { value: "want_to_read", "Want to Read" }
                                    option { value: "reading", "Reading" }
                                }
                            }
                        }
                    }

                    // Data
                    div {
                        h3 { class: SECTION_HEADING, "Data" }
                        div {
                            class: "{CARD} space-y-4",
                            div {
                                class: "flex items-center justify-between",
                                span { class: LABEL, "Database location" }
                                span {
                                    class: "max-w-sm truncate text-xs text-gray-500 dark:text-gray-400 font-mono",
                                    "{db_path}"
                                }
                            }
                            div {
                                class: "flex items-center justify-between",
                                span { class: LABEL, "Database size" }
                                span {
                                    class: "text-sm text-gray-500 dark:text-gray-400",
                                    "{db_size}"
                                }
                            }
                            // Stats row
                            div {
                                class: "flex gap-6 pt-2",
                                div {
                                    class: "text-center",
                                    div { class: STAT_VALUE, "{book_count}" }
                                    div { class: STAT_LABEL, "Books" }
                                }
                                div {
                                    class: "text-center",
                                    div { class: STAT_VALUE, "{diary_count}" }
                                    div { class: STAT_LABEL, "Diary Entries" }
                                }
                                div {
                                    class: "text-center",
                                    div { class: STAT_VALUE, "{highlight_count}" }
                                    div { class: STAT_LABEL, "Highlights" }
                                }
                            }
                        }
                    }

                    // About
                    div {
                        h3 { class: SECTION_HEADING, "About" }
                        div {
                            class: CARD,
                            div {
                                class: "flex items-center justify-between",
                                span { class: LABEL, "Version" }
                                span {
                                    class: "text-sm text-gray-500 dark:text-gray-400",
                                    {option_env!("CARGO_PKG_VERSION").unwrap_or("0.1.0")}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
