use dioxus::prelude::*;
use pulldown_cmark::{Options, Parser};

use crate::data::commands::{create_diary_entry_db, update_diary_entry_db};
use crate::data::models::DiaryEntry;
use crate::DatabaseHandle;

use super::rating_stars::RatingStars;

fn today_string() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = now / 86400;
    let mut y = 1970i32;
    let mut remaining_days = days as i32;

    loop {
        let year_days = if is_leap(y) { 366 } else { 365 };
        if remaining_days < year_days {
            break;
        }
        remaining_days -= year_days;
        y += 1;
    }

    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0u32;
    for &md in &month_days {
        if remaining_days < md {
            break;
        }
        remaining_days -= md;
        m += 1;
    }

    format!("{:04}-{:02}-{:02}", y, m + 1, remaining_days + 1)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn render_markdown(md: &str) -> String {
    let opts = Options::empty();
    let parser = Parser::new_ext(md, opts);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

#[derive(Props, Clone, PartialEq)]
pub struct DiaryEntryFormProps {
    book_id: i64,
    #[props(default)]
    book_title: Option<String>,
    #[props(default)]
    entry: Option<DiaryEntry>,
    on_save: EventHandler<()>,
    on_close: EventHandler<()>,
}

#[component]
pub fn DiaryEntryForm(props: DiaryEntryFormProps) -> Element {
    let db = use_context::<DatabaseHandle>();

    let starts_in_read = props.entry.is_some();
    let initial_date = props
        .entry
        .as_ref()
        .map(|e| e.entry_date.clone())
        .unwrap_or_else(today_string);
    let initial_rating = props.entry.as_ref().and_then(|e| e.rating);
    let initial_body = props
        .entry
        .as_ref()
        .and_then(|e| e.body.clone())
        .unwrap_or_default();
    let initial_id = props.entry.as_ref().map(|e| e.id);

    let mut entry_date = use_signal(|| initial_date);
    let mut rating: Signal<Option<i32>> = use_signal(|| initial_rating);
    let mut body = use_signal(|| initial_body);
    let mut entry_id: Signal<Option<i64>> = use_signal(|| initial_id);
    let mut save_status = use_signal(|| "idle");
    let mut is_read_mode = use_signal(|| starts_in_read);

    let preview_html = render_markdown(&body.read());

    let save = {
        let db = db.clone();
        let on_save = props.on_save;
        let book_id = props.book_id;
        move |_: ()| {
            let db = db.clone();
            let body_val = body.read().clone();
            let date_val = entry_date.read().clone();
            let rating_val = *rating.read();
            let eid = *entry_id.read();
            save_status.set("saving");
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                let body_opt = if body_val.is_empty() {
                    None
                } else {
                    Some(body_val.as_str())
                };
                let result = if let Some(id) = eid {
                    update_diary_entry_db(&conn, id, body_opt, rating_val, &date_val).map(|_| ())
                } else {
                    create_diary_entry_db(&conn, book_id, body_opt, rating_val, &date_val).map(
                        |created| {
                            entry_id.set(Some(created.id));
                        },
                    )
                };
                match result {
                    Ok(()) => {
                        save_status.set("saved");
                        on_save.call(());
                    }
                    Err(_) => {
                        save_status.set("idle");
                    }
                }
            });
        }
    };

    let on_close_handler = {
        let mut save = save.clone();
        let on_close = props.on_close;
        move |_: MouseEvent| {
            save(());
            on_close.call(());
        }
    };

    let save_click = {
        let mut save = save.clone();
        move |_: MouseEvent| {
            save(());
        }
    };

    fn append_syntax(body: &mut Signal<String>, prefix: &str, suffix: &str) {
        let current = body.read().clone();
        let new_val = format!("{current}{prefix}text{suffix}");
        body.set(new_val);
    }

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex flex-col bg-white dark:bg-gray-900",

            // Top bar
            div {
                class: "flex items-center gap-3 border-b border-gray-200 px-4 py-3 dark:border-gray-700",
                if *is_read_mode.read() {
                    button {
                        onclick: move |_| props.on_close.call(()),
                        class: "rounded-md p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-700
                            dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200",
                        aria_label: "Back",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "2",
                            stroke: "currentColor",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18",
                            }
                        }
                    }

                    div {
                        class: "flex min-w-0 flex-1 items-center justify-center gap-3",
                        if let Some(ref title) = props.book_title {
                            span {
                                class: "truncate text-sm font-medium text-gray-700 dark:text-gray-300",
                                "{title}"
                            }
                        }
                        span {
                            class: "text-sm text-gray-600 dark:text-gray-400",
                            "{entry_date}"
                        }
                        RatingStars {
                            rating: *rating.read(),
                            on_rate: move |_: i32| {},
                            small: true,
                        }
                    }

                    div { class: "w-20" }
                } else {
                    button {
                        onclick: on_close_handler,
                        class: "rounded-md p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-700
                            dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200",
                        aria_label: "Close",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "2",
                            stroke: "currentColor",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18",
                            }
                        }
                    }

                    div {
                        class: "flex min-w-0 flex-1 items-center justify-center gap-3",
                        if let Some(ref title) = props.book_title {
                            span {
                                class: "truncate text-sm font-medium text-gray-700 dark:text-gray-300",
                                "{title}"
                            }
                        }
                        input {
                            r#type: "date",
                            value: "{entry_date}",
                            oninput: move |e| entry_date.set(e.value()),
                            class: "rounded-md border border-gray-300 bg-white px-2 py-1 text-sm
                                text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
                                focus:ring-2 focus:ring-amber-500 focus:outline-none",
                        }
                        RatingStars {
                            rating: *rating.read(),
                            on_rate: move |score: i32| {
                                let new_val = if Some(score) == *rating.read() { None } else { Some(score) };
                                rating.set(new_val);
                            },
                            small: true,
                        }
                    }

                    div {
                        class: "w-20 text-right text-sm text-gray-400",
                        match *save_status.read() {
                            "saving" => rsx! { "Saving..." },
                            "saved" => rsx! {
                                span { class: "text-green-600 dark:text-green-400", "\u{2713} Saved" }
                            },
                            _ => rsx! {},
                        }
                    }
                }
            }

            if *is_read_mode.read() {
                // Full-width preview in read mode
                div {
                    class: "flex-1 overflow-y-auto bg-white dark:bg-gray-800/80",
                    div {
                        class: "mx-auto max-w-3xl px-10 py-12",
                        div {
                            class: "prose dark:prose-invert max-w-none text-sm
                                prose-headings:text-gray-800 dark:prose-headings:text-gray-200
                                prose-p:text-gray-700 dark:prose-p:text-gray-300",
                            dangerous_inner_html: "{preview_html}",
                        }
                    }
                }

                div {
                    class: "flex items-center gap-3 border-t border-gray-200 px-6 py-3 dark:border-gray-700",
                    button {
                        r#type: "button",
                        onclick: move |_| is_read_mode.set(false),
                        class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                            text-white transition hover:bg-amber-700 active:scale-95",
                        "Edit"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| props.on_close.call(()),
                        class: "rounded-lg border border-gray-300 px-5 py-2 text-sm font-medium
                            text-gray-700 transition hover:bg-gray-50 active:scale-95
                            dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-800",
                        "Back"
                    }
                }
            } else {
                // Markdown toolbar
                div {
                    class: "flex items-center gap-0.5 border-b border-gray-200 px-4 py-2 dark:border-gray-700",
                    button {
                        r#type: "button",
                        title: "Bold",
                        onclick: move |_| append_syntax(&mut body, "**", "**"),
                        class: "rounded-md p-2 text-gray-600 transition-colors hover:bg-gray-100
                            dark:text-gray-400 dark:hover:bg-gray-800",
                        span { class: "text-sm font-bold", "B" }
                    }
                    button {
                        r#type: "button",
                        title: "Italic",
                        onclick: move |_| append_syntax(&mut body, "*", "*"),
                        class: "rounded-md p-2 text-gray-600 transition-colors hover:bg-gray-100
                            dark:text-gray-400 dark:hover:bg-gray-800",
                        span { class: "text-sm italic", "I" }
                    }
                    button {
                        r#type: "button",
                        title: "Heading",
                        onclick: move |_| append_syntax(&mut body, "## ", ""),
                        class: "rounded-md p-2 text-gray-600 transition-colors hover:bg-gray-100
                            dark:text-gray-400 dark:hover:bg-gray-800",
                        span { class: "text-sm font-medium", "H" }
                    }
                    button {
                        r#type: "button",
                        title: "List",
                        onclick: move |_| append_syntax(&mut body, "- ", ""),
                        class: "rounded-md p-2 text-gray-600 transition-colors hover:bg-gray-100
                            dark:text-gray-400 dark:hover:bg-gray-800",
                        span { class: "text-sm font-medium", "-" }
                    }
                    button {
                        r#type: "button",
                        title: "Quote",
                        onclick: move |_| append_syntax(&mut body, "> ", ""),
                        class: "rounded-md p-2 text-gray-600 transition-colors hover:bg-gray-100
                            dark:text-gray-400 dark:hover:bg-gray-800",
                        span { class: "text-sm font-medium", ">" }
                    }
                }

                // Editor and preview
                div {
                    class: "flex flex-1 overflow-hidden",

                    // Markdown editor
                    div {
                        class: "flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900",
                        div {
                            class: "mx-auto max-w-3xl px-6 py-6",
                            textarea {
                                value: "{body}",
                                oninput: move |e| body.set(e.value()),
                                placeholder: "Write your thoughts in Markdown...",
                                class: "min-h-[60vh] w-full resize-none rounded-xl bg-white px-10 py-12
                                    text-sm leading-relaxed text-gray-700 shadow-sm ring-1
                                    ring-gray-200/50 placeholder-gray-400 outline-none
                                    dark:bg-gray-800/80 dark:text-gray-300 dark:ring-gray-700/50
                                    dark:placeholder-gray-500",
                            }
                        }
                    }

                    // Live preview
                    div {
                        class: "flex-1 overflow-y-auto border-l border-gray-200 bg-white
                            dark:border-gray-700 dark:bg-gray-800/80",
                        div {
                            class: "mx-auto max-w-3xl px-10 py-12",
                            div {
                                class: "prose dark:prose-invert max-w-none text-sm
                                    prose-headings:text-gray-800 dark:prose-headings:text-gray-200
                                    prose-p:text-gray-700 dark:prose-p:text-gray-300",
                                dangerous_inner_html: "{preview_html}",
                            }
                        }
                    }
                }

                // Save button
                div {
                    class: "border-t border-gray-200 px-6 py-3 dark:border-gray-700",
                    button {
                        r#type: "button",
                        onclick: save_click,
                        class: "rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                            text-white transition hover:bg-amber-700 active:scale-95",
                        "Save Entry"
                    }
                }
            }
        }
    }
}
