use dioxus::prelude::*;

use crate::data::models::Book;

fn status_color(status: &str) -> &'static str {
    match status {
        "want_to_read" => "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300",
        "reading" => "bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300",
        "finished" => "bg-purple-100 text-purple-800 dark:bg-purple-900/40 dark:text-purple-300",
        "abandoned" => "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
        _ => "",
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "want_to_read" => "Want to Read",
        "reading" => "Reading",
        "finished" => "Finished",
        "abandoned" => "Abandoned",
        _ => "",
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct LibraryListProps {
    books: Vec<Book>,
    on_select_book: EventHandler<i64>,
}

#[component]
pub fn LibraryList(props: LibraryListProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 p-6",
            for book in props.books.iter() {
                {
                    let first_char = book.title.chars().next().unwrap_or('?').to_uppercase().to_string();
                    let status = book.status.as_deref().unwrap_or("");
                    let has_status = !status.is_empty();
                    let color = status_color(status);
                    let label = status_label(status);
                    let date = book.created_at.split('T').next().unwrap_or(&book.created_at).to_string();
                    let stars: String = if let Some(r) = book.rating {
                        (1..=5).map(|i| if i <= r { '\u{2605}' } else { '\u{2606}' }).collect()
                    } else {
                        String::new()
                    };
                    let id = book.id;
                    rsx! {
                        button {
                            key: "{id}",
                            r#type: "button",
                            onclick: move |_| props.on_select_book.call(id),
                            class: "flex items-center gap-4 rounded-lg px-3 py-2 text-left
                                transition hover:bg-gray-100 dark:hover:bg-gray-800/60 cursor-pointer
                                focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none",

                            // Thumbnail
                            div {
                                class: "h-14 w-10 flex-shrink-0 overflow-hidden rounded bg-gray-100 dark:bg-gray-700",
                                if let Some(ref cover_url) = book.cover_url {
                                    img {
                                        src: "{cover_url}",
                                        alt: "{book.title}",
                                        class: "h-full w-full object-cover",
                                    }
                                } else {
                                    div {
                                        class: "flex h-full w-full items-center justify-center
                                            bg-gradient-to-br from-amber-100 to-orange-200
                                            dark:from-amber-900/40 dark:to-orange-900/40",
                                        span {
                                            class: "text-sm font-bold text-amber-700/60 dark:text-amber-400/60",
                                            "{first_char}"
                                        }
                                    }
                                }
                            }

                            // Title & author
                            div {
                                class: "min-w-0 flex-1",
                                div {
                                    class: "truncate text-sm font-semibold text-gray-900 dark:text-gray-100",
                                    "{book.title}"
                                }
                                if let Some(ref author) = book.author {
                                    div {
                                        class: "truncate text-sm text-gray-500 dark:text-gray-400",
                                        "{author}"
                                    }
                                }
                            }

                            // Status badge
                            if has_status {
                                span {
                                    class: "flex-shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium {color}",
                                    "{label}"
                                }
                            }

                            // Rating
                            if !stars.is_empty() {
                                div {
                                    class: "flex-shrink-0",
                                    span {
                                        class: "text-xs text-amber-500",
                                        "{stars}"
                                    }
                                }
                            }

                            // Date added
                            span {
                                class: "flex-shrink-0 text-xs text-gray-400 dark:text-gray-500",
                                "{date}"
                            }
                        }
                    }
                }
            }
        }
    }
}
