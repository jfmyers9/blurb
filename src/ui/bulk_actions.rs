use std::collections::HashSet;

use dioxus::prelude::*;

use crate::data::commands::{
    bulk_add_to_shelf_db, bulk_delete_books_db, bulk_set_rating_db, bulk_set_status_db,
};
use crate::data::models::Shelf;
use crate::DatabaseHandle;

#[derive(Clone, Copy, PartialEq)]
enum OpenDropdown {
    None,
    Status,
    Rating,
    Shelf,
    Delete,
}

#[derive(Props, Clone, PartialEq)]
pub struct BulkActionsProps {
    selected_ids: Signal<HashSet<i64>>,
    all_book_ids: Vec<i64>,
    shelves: Vec<Shelf>,
    on_done: EventHandler<()>,
}

#[component]
pub fn BulkActions(props: BulkActionsProps) -> Element {
    let count = props.selected_ids.read().len();
    if count == 0 {
        return rsx! {};
    }

    let db = use_context::<DatabaseHandle>();
    let mut dropdown = use_signal(|| OpenDropdown::None);

    let toggle = move |target: OpenDropdown| {
        let current = *dropdown.read();
        if current == target {
            dropdown.set(OpenDropdown::None);
        } else {
            dropdown.set(target);
        }
    };

    let total = props.all_book_ids.len();
    let all_selected = count == total && total > 0;

    rsx! {
        div {
            class: "flex items-center gap-3 rounded-lg bg-amber-50 px-4 py-2
                ring-1 ring-amber-200 dark:bg-amber-900/20 dark:ring-amber-800",

            span {
                class: "text-sm font-medium text-amber-800 dark:text-amber-300",
                "{count} selected"
            }

            button {
                r#type: "button",
                onclick: {
                    let all_ids = props.all_book_ids.clone();
                    let mut selected_ids = props.selected_ids;
                    move |_| {
                        if all_selected {
                            selected_ids.set(HashSet::new());
                        } else {
                            selected_ids.set(all_ids.iter().copied().collect());
                        }
                    }
                },
                class: "text-xs text-amber-600 underline hover:text-amber-800
                    dark:text-amber-400 dark:hover:text-amber-200",
                if all_selected { "Deselect All" } else { "Select All" }
            }

            div { class: "mx-1 h-4 w-px bg-amber-300 dark:bg-amber-700" }

            div {
                class: "relative",
                button {
                    r#type: "button",
                    onclick: move |_| toggle(OpenDropdown::Status),
                    class: "rounded-md bg-white px-3 py-1 text-xs font-medium text-gray-700
                        shadow-sm ring-1 ring-gray-300 hover:bg-gray-50
                        dark:bg-gray-800 dark:text-gray-200 dark:ring-gray-600 dark:hover:bg-gray-700",
                    "Set Status"
                }
                if *dropdown.read() == OpenDropdown::Status {
                    {StatusDropdown(StatusDropdownProps {
                        selected_ids: props.selected_ids,
                        db: db.clone(),
                        on_done: props.on_done,
                        on_close: EventHandler::new(move |_| dropdown.set(OpenDropdown::None)),
                    })}
                }
            }

            div {
                class: "relative",
                button {
                    r#type: "button",
                    onclick: move |_| toggle(OpenDropdown::Rating),
                    class: "rounded-md bg-white px-3 py-1 text-xs font-medium text-gray-700
                        shadow-sm ring-1 ring-gray-300 hover:bg-gray-50
                        dark:bg-gray-800 dark:text-gray-200 dark:ring-gray-600 dark:hover:bg-gray-700",
                    "Set Rating"
                }
                if *dropdown.read() == OpenDropdown::Rating {
                    {RatingDropdown(RatingDropdownProps {
                        selected_ids: props.selected_ids,
                        db: db.clone(),
                        on_done: props.on_done,
                        on_close: EventHandler::new(move |_| dropdown.set(OpenDropdown::None)),
                    })}
                }
            }

            div {
                class: "relative",
                button {
                    r#type: "button",
                    onclick: move |_| toggle(OpenDropdown::Shelf),
                    class: "rounded-md bg-white px-3 py-1 text-xs font-medium text-gray-700
                        shadow-sm ring-1 ring-gray-300 hover:bg-gray-50
                        dark:bg-gray-800 dark:text-gray-200 dark:ring-gray-600 dark:hover:bg-gray-700",
                    "Add to Shelf"
                }
                if *dropdown.read() == OpenDropdown::Shelf {
                    {ShelfDropdown(ShelfDropdownProps {
                        selected_ids: props.selected_ids,
                        shelves: props.shelves.clone(),
                        db: db.clone(),
                        on_done: props.on_done,
                        on_close: EventHandler::new(move |_| dropdown.set(OpenDropdown::None)),
                    })}
                }
            }

            div {
                class: "relative",
                button {
                    r#type: "button",
                    onclick: move |_| toggle(OpenDropdown::Delete),
                    class: "rounded-md bg-red-50 px-3 py-1 text-xs font-medium text-red-700
                        shadow-sm ring-1 ring-red-200 hover:bg-red-100
                        dark:bg-red-900/30 dark:text-red-300 dark:ring-red-800 dark:hover:bg-red-900/50",
                    "Delete"
                }
                if *dropdown.read() == OpenDropdown::Delete {
                    {DeleteConfirm(DeleteConfirmProps {
                        selected_ids: props.selected_ids,
                        db: db.clone(),
                        on_done: props.on_done,
                        on_close: EventHandler::new(move |_| dropdown.set(OpenDropdown::None)),
                    })}
                }
            }

            div { class: "flex-1" }

            button {
                r#type: "button",
                onclick: {
                    let mut selected_ids = props.selected_ids;
                    move |_| selected_ids.set(HashSet::new())
                },
                class: "text-xs text-gray-500 hover:text-gray-700
                    dark:text-gray-400 dark:hover:text-gray-200",
                "Clear selection"
            }
        }
    }
}

const DROPDOWN_CLASS: &str = "absolute left-0 top-full z-20 mt-1 min-w-[140px] rounded-md \
    border border-gray-200 bg-white py-1 shadow-lg \
    dark:border-gray-700 dark:bg-gray-800";

const DROPDOWN_ITEM: &str = "block w-full px-3 py-1.5 text-left text-xs text-gray-700 \
    hover:bg-amber-50 dark:text-gray-200 dark:hover:bg-amber-900/20";

#[derive(Props, Clone, PartialEq)]
struct StatusDropdownProps {
    selected_ids: Signal<HashSet<i64>>,
    db: DatabaseHandle,
    on_done: EventHandler<()>,
    on_close: EventHandler<()>,
}

fn StatusDropdown(props: StatusDropdownProps) -> Element {
    let statuses = [
        ("want_to_read", "Want to Read"),
        ("reading", "Reading"),
        ("finished", "Finished"),
        ("abandoned", "Abandoned"),
    ];

    rsx! {
        div {
            class: DROPDOWN_CLASS,
            for (value, label) in statuses {
                button {
                    r#type: "button",
                    onclick: {
                        let db = props.db.clone();
                        let selected_ids = props.selected_ids;
                        let on_done = props.on_done;
                        let on_close = props.on_close;
                        move |_| {
                            let db = db.clone();
                            let ids: Vec<i64> = selected_ids.read().iter().copied().collect();
                            spawn(async move {
                                let mut conn = db.conn.lock().unwrap();
                                let _ = bulk_set_status_db(&mut conn, &ids, value);
                                drop(conn);
                                on_close.call(());
                                on_done.call(());
                            });
                        }
                    },
                    class: DROPDOWN_ITEM,
                    "{label}"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct RatingDropdownProps {
    selected_ids: Signal<HashSet<i64>>,
    db: DatabaseHandle,
    on_done: EventHandler<()>,
    on_close: EventHandler<()>,
}

fn RatingDropdown(props: RatingDropdownProps) -> Element {
    rsx! {
        div {
            class: DROPDOWN_CLASS,
            for score in 1..=5 {
                button {
                    r#type: "button",
                    onclick: {
                        let db = props.db.clone();
                        let selected_ids = props.selected_ids;
                        let on_done = props.on_done;
                        let on_close = props.on_close;
                        move |_| {
                            let db = db.clone();
                            let ids: Vec<i64> = selected_ids.read().iter().copied().collect();
                            spawn(async move {
                                let mut conn = db.conn.lock().unwrap();
                                let _ = bulk_set_rating_db(&mut conn, &ids, score);
                                drop(conn);
                                on_close.call(());
                                on_done.call(());
                            });
                        }
                    },
                    class: DROPDOWN_ITEM,
                    {(1..=5).map(|i| if i <= score { "\u{2605}" } else { "\u{2606}" }).collect::<String>()}
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ShelfDropdownProps {
    selected_ids: Signal<HashSet<i64>>,
    shelves: Vec<Shelf>,
    db: DatabaseHandle,
    on_done: EventHandler<()>,
    on_close: EventHandler<()>,
}

fn ShelfDropdown(props: ShelfDropdownProps) -> Element {
    if props.shelves.is_empty() {
        return rsx! {
            div {
                class: DROPDOWN_CLASS,
                div {
                    class: "px-3 py-2 text-xs text-gray-500 dark:text-gray-400",
                    "No shelves yet"
                }
            }
        };
    }

    rsx! {
        div {
            class: DROPDOWN_CLASS,
            for shelf in props.shelves.iter() {
                button {
                    key: "{shelf.id}",
                    r#type: "button",
                    onclick: {
                        let db = props.db.clone();
                        let selected_ids = props.selected_ids;
                        let shelf_id = shelf.id;
                        let on_done = props.on_done;
                        let on_close = props.on_close;
                        move |_| {
                            let db = db.clone();
                            let ids: Vec<i64> = selected_ids.read().iter().copied().collect();
                            spawn(async move {
                                let mut conn = db.conn.lock().unwrap();
                                let _ = bulk_add_to_shelf_db(&mut conn, &ids, shelf_id);
                                drop(conn);
                                on_close.call(());
                                on_done.call(());
                            });
                        }
                    },
                    class: DROPDOWN_ITEM,
                    "{shelf.name}"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DeleteConfirmProps {
    selected_ids: Signal<HashSet<i64>>,
    db: DatabaseHandle,
    on_done: EventHandler<()>,
    on_close: EventHandler<()>,
}

fn DeleteConfirm(props: DeleteConfirmProps) -> Element {
    let count = props.selected_ids.read().len();

    rsx! {
        div {
            class: "absolute left-0 top-full z-20 mt-1 w-56 rounded-md
                border border-red-200 bg-white p-3 shadow-lg
                dark:border-red-800 dark:bg-gray-800",
            p {
                class: "mb-2 text-xs text-gray-700 dark:text-gray-300",
                "Delete {count} book{if count != 1 { \"s\" } else { \"\" }}? This cannot be undone."
            }
            div {
                class: "flex gap-2",
                button {
                    r#type: "button",
                    onclick: {
                        let on_close = props.on_close;
                        move |_| on_close.call(())
                    },
                    class: "flex-1 rounded-md bg-gray-100 px-2 py-1 text-xs font-medium
                        text-gray-700 hover:bg-gray-200
                        dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600",
                    "Cancel"
                }
                button {
                    r#type: "button",
                    onclick: {
                        let db = props.db.clone();
                        let mut selected_ids = props.selected_ids;
                        let on_done = props.on_done;
                        let on_close = props.on_close;
                        move |_| {
                            let db = db.clone();
                            let ids: Vec<i64> = selected_ids.read().iter().copied().collect();
                            spawn(async move {
                                let mut conn = db.conn.lock().unwrap();
                                let _ = bulk_delete_books_db(&mut conn, &ids);
                                drop(conn);
                                selected_ids.set(HashSet::new());
                                on_close.call(());
                                on_done.call(());
                            });
                        }
                    },
                    class: "flex-1 rounded-md bg-red-600 px-2 py-1 text-xs font-medium
                        text-white hover:bg-red-700",
                    "Delete"
                }
            }
        }
    }
}
