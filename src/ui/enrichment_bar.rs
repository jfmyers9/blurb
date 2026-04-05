use dioxus::prelude::*;

use crate::services::enrichment::{EnrichmentState, EnrichmentStatus};

#[component]
pub fn EnrichmentBar() -> Element {
    let mut state = use_context::<EnrichmentState>();
    let status = state.status.read().clone();

    match status {
        EnrichmentStatus::Idle => rsx! {},
        EnrichmentStatus::Running {
            current,
            total,
            current_title,
        } => {
            let pct = if total > 0 {
                (current as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            rsx! {
                div {
                    class: "fixed bottom-0 inset-x-0 z-40 flex items-center gap-3 border-t
                        border-gray-200 bg-white/90 px-4 py-2 text-sm backdrop-blur
                        dark:border-gray-700 dark:bg-gray-900/90",
                    div {
                        class: "h-1.5 flex-1 overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700",
                        div {
                            class: "h-full rounded-full bg-amber-500 transition-all duration-300",
                            style: "width: {pct:.1}%",
                        }
                    }
                    span {
                        class: "shrink-0 text-gray-600 dark:text-gray-400",
                        "Enriching {current}/{total} — {current_title}"
                    }
                }
            }
        }
        EnrichmentStatus::Done { succeeded, failed } => {
            use_future(move || async move {
                if failed == 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    state.status.set(EnrichmentStatus::Idle);
                }
            });

            let msg = if failed > 0 {
                format!("Enriched {succeeded} books, {failed} failed")
            } else {
                format!("Enriched {succeeded} books")
            };
            let bar_border = if failed > 0 {
                "border-red-300 dark:border-red-700"
            } else {
                "border-green-300 dark:border-green-700"
            };

            rsx! {
                div {
                    class: "fixed bottom-0 inset-x-0 z-40 flex items-center justify-between
                        border-t bg-white/90 px-4 py-2 text-sm backdrop-blur
                        dark:bg-gray-900/90 {bar_border}",
                    span {
                        class: "text-gray-700 dark:text-gray-300",
                        "{msg}"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| state.status.set(EnrichmentStatus::Idle),
                        class: "rounded px-2 py-0.5 text-xs text-gray-500 hover:bg-gray-100
                            dark:text-gray-400 dark:hover:bg-gray-800",
                        "Dismiss"
                    }
                }
            }
        }
    }
}
