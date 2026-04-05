use dioxus::prelude::*;

use crate::data::commands::{
    get_reading_goal_progress_db, list_reading_goals_db, set_reading_goal_db,
};
use crate::data::models::ReadingGoalProgress;
use crate::DatabaseHandle;

fn current_year() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days_since_epoch = secs / 86400;
    let mut year = 1970i32;
    let mut remaining_days = days_since_epoch;
    loop {
        let days_in_year: u64 = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    year
}

fn progress_color(progress: &ReadingGoalProgress) -> &'static str {
    if progress.on_track {
        "bg-green-500"
    } else if progress.percent_complete >= 60.0 {
        "bg-amber-500"
    } else {
        "bg-red-500"
    }
}

fn progress_text_color(progress: &ReadingGoalProgress) -> &'static str {
    if progress.on_track {
        "text-green-600 dark:text-green-400"
    } else if progress.percent_complete >= 60.0 {
        "text-amber-600 dark:text-amber-400"
    } else {
        "text-red-600 dark:text-red-400"
    }
}

#[component]
fn GoalProgressCard(progress: ReadingGoalProgress, year: i32) -> Element {
    let mut editing = use_signal(|| false);
    let mut goal_input = use_signal(String::new);
    let bar_color = progress_color(&progress);
    let text_color = progress_text_color(&progress);
    let pct = progress.percent_complete.min(100.0);
    let target = progress.goal.target_books;
    let finished = progress.books_finished;

    rsx! {
        div {
            class: "mb-4",
            div {
                class: "flex items-center justify-between mb-2",
                span {
                    class: "text-sm font-medium text-gray-700 dark:text-gray-300",
                    "{year} Goal"
                }
                if *editing.read() {
                    EditGoalInput {
                        year: year,
                        goal_input: goal_input,
                        editing: editing,
                    }
                } else {
                    button {
                        class: "text-xs px-2 py-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200",
                        onclick: move |_| {
                            goal_input.set(target.to_string());
                            editing.set(true);
                        },
                        "Edit goal"
                    }
                }
            }
            div {
                class: "w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 mb-2",
                div {
                    class: "h-3 rounded-full transition-all {bar_color}",
                    style: "width: {pct}%",
                }
            }
            div {
                class: "flex items-center justify-between",
                span {
                    class: "text-sm font-medium {text_color}",
                    "{finished} of {target} books"
                }
                span {
                    class: "text-sm {text_color}",
                    "{pct:.0}%"
                }
            }
        }
    }
}

#[component]
fn EditGoalInput(year: i32, goal_input: Signal<String>, editing: Signal<bool>) -> Element {
    let db = use_context::<DatabaseHandle>();

    let save_goal = {
        let db = db.clone();
        move || {
            let input = goal_input.read().clone();
            if let Ok(target) = input.trim().parse::<i32>() {
                if target > 0 {
                    let db = db.clone();
                    spawn(async move {
                        let conn = db.conn.lock().unwrap();
                        let _ = set_reading_goal_db(&conn, year, target);
                        drop(conn);
                        editing.set(false);
                    });
                }
            }
        }
    };

    rsx! {
        div {
            class: "flex items-center gap-2",
            input {
                class: "w-20 px-2 py-1 text-sm border rounded dark:bg-gray-700 dark:border-gray-600 dark:text-white",
                r#type: "number",
                min: "1",
                value: "{goal_input}",
                oninput: move |e| goal_input.set(e.value()),
                onkeydown: {
                    let save = save_goal.clone();
                    move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            save();
                        }
                    }
                },
            }
            button {
                class: "text-xs px-2 py-1 bg-blue-500 text-white rounded hover:bg-blue-600",
                onclick: {
                    let save = save_goal.clone();
                    move |_| save()
                },
                "Save"
            }
            button {
                class: "text-xs px-2 py-1 text-gray-500 hover:text-gray-700 dark:text-gray-400",
                onclick: move |_| editing.set(false),
                "Cancel"
            }
        }
    }
}

#[component]
fn SetGoalPrompt(year: i32) -> Element {
    let mut editing = use_signal(|| false);
    let mut goal_input = use_signal(String::new);

    rsx! {
        div {
            class: "text-center py-4",
            if *editing.read() {
                EditGoalInput {
                    year: year,
                    goal_input: goal_input,
                    editing: editing,
                }
            } else {
                button {
                    class: "text-sm text-blue-500 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300",
                    onclick: move |_| {
                        goal_input.set(String::new());
                        editing.set(true);
                    },
                    "Set a reading goal for {year}"
                }
            }
        }
    }
}

#[component]
fn PastGoalRow(progress: ReadingGoalProgress) -> Element {
    let bar_color = progress_color(&progress);
    let text_color = progress_text_color(&progress);
    let pct = progress.percent_complete.min(100.0);
    let year = progress.goal.year;
    let finished = progress.books_finished;
    let target = progress.goal.target_books;

    rsx! {
        div {
            class: "flex items-center justify-between py-1",
            span {
                class: "text-sm text-gray-600 dark:text-gray-300",
                "{year}"
            }
            div {
                class: "flex items-center gap-2",
                div {
                    class: "w-24 bg-gray-200 dark:bg-gray-700 rounded-full h-1.5",
                    div {
                        class: "h-1.5 rounded-full {bar_color}",
                        style: "width: {pct}%",
                    }
                }
                span {
                    class: "text-xs {text_color}",
                    "{finished}/{target}"
                }
            }
        }
    }
}

#[component]
pub fn ReadingGoals() -> Element {
    let db = use_context::<DatabaseHandle>();
    let year = current_year();

    let mut current_progress: Signal<Option<ReadingGoalProgress>> = use_signal(|| None);
    let mut past_goals: Signal<Vec<ReadingGoalProgress>> = use_signal(Vec::new);

    let load_goals = {
        let db = db.clone();
        move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                if let Ok(progress) = get_reading_goal_progress_db(&conn, year) {
                    current_progress.set(progress);
                }
                if let Ok(all) = list_reading_goals_db(&conn) {
                    let others: Vec<ReadingGoalProgress> =
                        all.into_iter().filter(|p| p.goal.year != year).collect();
                    past_goals.set(others);
                }
            });
        }
    };

    use_effect({
        let load = load_goals.clone();
        move || {
            load();
        }
    });

    let cp = current_progress.read().clone();
    let pg = past_goals.read().clone();

    rsx! {
        div {
            class: "bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-5",

            h3 {
                class: "text-lg font-semibold text-gray-900 dark:text-white mb-4",
                "Reading Goals"
            }

            if let Some(progress) = cp {
                GoalProgressCard { progress: progress, year: year }
            } else {
                SetGoalPrompt { year: year }
            }

            if !pg.is_empty() {
                div {
                    class: "border-t border-gray-200 dark:border-gray-700 pt-3 mt-3",
                    h4 {
                        class: "text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide mb-2",
                        "Previous Years"
                    }
                    for past in pg.iter() {
                        PastGoalRow { key: "{past.goal.year}", progress: past.clone() }
                    }
                }
            }
        }
    }
}
