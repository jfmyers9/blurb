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
pub struct BookCardProps {
    book: Book,
    on_click: EventHandler<i64>,
}

#[component]
pub fn BookCard(props: BookCardProps) -> Element {
    let book = &props.book;
    let first_char = book
        .title
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let has_status = book.status.as_deref().is_some_and(|s| !s.is_empty());

    rsx! {
        button {
            r#type: "button",
            onclick: {
                let id = book.id;
                move |_| props.on_click.call(id)
            },
            class: "group flex flex-col overflow-hidden rounded-lg bg-white
                shadow-sm ring-1 ring-gray-200 transition hover:shadow-md
                hover:ring-amber-300 dark:bg-gray-800 dark:ring-gray-700
                dark:hover:ring-amber-600 cursor-pointer text-left
                focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none",

            // Cover
            div {
                class: "relative aspect-[2/3] w-full overflow-hidden bg-gray-100 dark:bg-gray-700",
                if let Some(ref cover_url) = book.cover_url {
                    img {
                        src: "{cover_url}",
                        alt: "{book.title}",
                        class: "h-full w-full object-cover transition group-hover:scale-105",
                    }
                } else {
                    div {
                        class: "flex h-full w-full items-center justify-center
                            bg-gradient-to-br from-amber-100 to-orange-200
                            dark:from-amber-900/40 dark:to-orange-900/40",
                        span {
                            class: "text-4xl font-bold text-amber-700/60 dark:text-amber-400/60",
                            "{first_char}"
                        }
                    }
                }
                if has_status {
                    {
                        let status = book.status.as_deref().unwrap_or("");
                        let color = status_color(status);
                        let label = status_label(status);
                        rsx! {
                            span {
                                class: "absolute top-2 right-2 rounded-full px-2 py-0.5 text-[10px] font-medium {color}",
                                "{label}"
                            }
                        }
                    }
                }
            }

            // Info
            div {
                class: "flex flex-1 flex-col gap-1 p-3",
                h3 {
                    class: "line-clamp-2 text-sm font-semibold text-gray-900 dark:text-gray-100",
                    "{book.title}"
                }
                if let Some(ref author) = book.author {
                    p {
                        class: "line-clamp-1 text-xs text-gray-500 dark:text-gray-400",
                        "{author}"
                    }
                }
                if let Some(rating) = book.rating {
                    div {
                        class: "mt-auto pt-1",
                        RatingStars { rating }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct RatingStarsProps {
    rating: i32,
}

#[component]
fn RatingStars(props: RatingStarsProps) -> Element {
    let stars: String = (1..=5)
        .map(|i| {
            if i <= props.rating {
                '\u{2605}'
            } else {
                '\u{2606}'
            }
        })
        .collect();
    rsx! {
        span {
            class: "text-xs text-amber-500",
            "{stars}"
        }
    }
}
