use dioxus::prelude::*;

use crate::data::commands::get_reading_stats_db;
use crate::data::models::ReadingStats;
use crate::DatabaseHandle;

#[component]
pub fn StatsView() -> Element {
    let db = use_context::<DatabaseHandle>();
    let mut stats: Signal<Option<ReadingStats>> = use_signal(|| None);

    use_effect(move || {
        let db = db.clone();
        spawn(async move {
            let conn = db.conn.lock().unwrap();
            if let Ok(s) = get_reading_stats_db(&conn) {
                stats.set(Some(s));
            }
        });
    });

    let s = stats.read();
    let Some(s) = s.as_ref() else {
        return rsx! {
            div {
                class: "flex items-center justify-center py-24",
                div {
                    class: "text-sm text-gray-400 dark:text-gray-500",
                    "Loading statistics..."
                }
            }
        };
    };

    let max_per_month = s.books_per_month.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let max_rating_dist = s.rating_distribution.iter().copied().max().unwrap_or(1);

    rsx! {
        div {
            class: "mx-auto max-w-4xl px-6 py-8",
            h2 {
                class: "mb-6 text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100",
                "Reading Statistics"
            }

            // Stat cards row
            div {
                class: "mb-8 grid grid-cols-2 gap-4 sm:grid-cols-4",
                StatCard { label: "Total Books", value: s.total_books.to_string() }
                StatCard { label: "Finished", value: s.books_finished.to_string() }
                StatCard {
                    label: "Pages Read",
                    value: format_number(s.total_pages_read),
                }
                StatCard {
                    label: "Avg Rating",
                    value: match s.avg_rating {
                        Some(r) => format!("{r:.1}"),
                        None => "--".to_string(),
                    },
                }
            }

            // Secondary stats
            div {
                class: "mb-8 grid grid-cols-2 gap-4 sm:grid-cols-5",
                MiniStat { label: "Reading", value: s.books_reading.to_string() }
                MiniStat { label: "Want to Read", value: s.books_want_to_read.to_string() }
                MiniStat { label: "Abandoned", value: s.books_abandoned.to_string() }
                MiniStat { label: "Diary Entries", value: s.total_diary_entries.to_string() }
                MiniStat { label: "Highlights", value: s.total_highlights.to_string() }
            }

            // Two-column layout for charts
            div {
                class: "mb-8 grid gap-6 lg:grid-cols-2",

                // Books per month
                div {
                    class: "rounded-xl border border-gray-200 bg-white p-5 dark:border-gray-800 dark:bg-gray-900",
                    h3 {
                        class: "mb-4 text-sm font-semibold text-gray-700 dark:text-gray-300",
                        "Books Finished per Month"
                    }
                    if s.books_per_month.is_empty() {
                        p {
                            class: "text-sm text-gray-400 dark:text-gray-500",
                            "No finished books yet."
                        }
                    } else {
                        div {
                            class: "space-y-2",
                            for (month, count) in s.books_per_month.iter().rev() {
                                div {
                                    class: "flex items-center gap-3",
                                    span {
                                        class: "w-16 shrink-0 text-xs text-gray-500 dark:text-gray-400",
                                        "{month}"
                                    }
                                    div {
                                        class: "relative h-5 flex-1 overflow-hidden rounded bg-gray-100 dark:bg-gray-800",
                                        div {
                                            class: "h-full rounded bg-amber-500 dark:bg-amber-600 transition-all",
                                            style: "width: {pct(*count, max_per_month)}%",
                                        }
                                    }
                                    span {
                                        class: "w-6 text-right text-xs font-medium text-gray-600 dark:text-gray-300",
                                        "{count}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Rating distribution
                div {
                    class: "rounded-xl border border-gray-200 bg-white p-5 dark:border-gray-800 dark:bg-gray-900",
                    h3 {
                        class: "mb-4 text-sm font-semibold text-gray-700 dark:text-gray-300",
                        "Rating Distribution"
                    }
                    div {
                        class: "flex items-end gap-3 h-32",
                        for i in 0..5 {
                            {
                                let count = s.rating_distribution[i];
                                let height = pct(count, max_rating_dist);
                                let stars = i + 1;
                                rsx! {
                                    div {
                                        class: "flex flex-1 flex-col items-center gap-1",
                                        span {
                                            class: "text-xs font-medium text-gray-600 dark:text-gray-300",
                                            "{count}"
                                        }
                                        div {
                                            class: "w-full rounded-t bg-amber-500 dark:bg-amber-600 transition-all",
                                            style: "height: {height}%; min-height: 2px",
                                        }
                                        span {
                                            class: "text-xs text-gray-500 dark:text-gray-400",
                                            "{stars}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Top rated books
            if !s.top_rated_books.is_empty() {
                div {
                    class: "mb-8 rounded-xl border border-gray-200 bg-white p-5 dark:border-gray-800 dark:bg-gray-900",
                    h3 {
                        class: "mb-4 text-sm font-semibold text-gray-700 dark:text-gray-300",
                        "Top Rated Books"
                    }
                    div {
                        class: "space-y-2",
                        for (title, author, _rating) in s.top_rated_books.iter() {
                            div {
                                class: "flex items-center justify-between",
                                div {
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
                                span {
                                    class: "text-amber-500",
                                    "\u{2605}\u{2605}\u{2605}\u{2605}\u{2605}"
                                }
                            }
                        }
                    }
                }
            }

            // Recent activity
            if !s.recent_activity.is_empty() {
                div {
                    class: "rounded-xl border border-gray-200 bg-white p-5 dark:border-gray-800 dark:bg-gray-900",
                    h3 {
                        class: "mb-4 text-sm font-semibold text-gray-700 dark:text-gray-300",
                        "Recent Activity"
                    }
                    div {
                        class: "space-y-2",
                        for (date, action, book_title) in s.recent_activity.iter() {
                            div {
                                class: "flex items-center gap-3 text-sm",
                                span {
                                    class: "w-24 shrink-0 text-xs text-gray-400 dark:text-gray-500",
                                    "{date}"
                                }
                                span {
                                    class: "rounded bg-amber-100 px-1.5 py-0.5 text-xs font-medium text-amber-800 dark:bg-amber-900/40 dark:text-amber-300",
                                    "{action}"
                                }
                                span {
                                    class: "truncate text-gray-700 dark:text-gray-300",
                                    "{book_title}"
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
fn StatCard(label: String, value: String) -> Element {
    rsx! {
        div {
            class: "rounded-xl border border-gray-200 bg-white p-4 dark:border-gray-800 dark:bg-gray-900",
            p {
                class: "text-xs font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400",
                "{label}"
            }
            p {
                class: "mt-1 text-2xl font-bold text-gray-900 dark:text-gray-100",
                "{value}"
            }
        }
    }
}

#[component]
fn MiniStat(label: String, value: String) -> Element {
    rsx! {
        div {
            class: "rounded-lg border border-gray-200 bg-white px-3 py-2 dark:border-gray-800 dark:bg-gray-900",
            p {
                class: "text-xs text-gray-500 dark:text-gray-400",
                "{label}"
            }
            p {
                class: "text-lg font-semibold text-gray-800 dark:text-gray-200",
                "{value}"
            }
        }
    }
}

fn pct(value: usize, max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    (value * 100) / max
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
