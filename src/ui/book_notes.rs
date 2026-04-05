use dioxus::prelude::*;

use crate::data::commands::{
    create_book_note_db, delete_book_note_db, list_book_notes_db, update_book_note_db,
};
use crate::data::models::BookNote;
use crate::DatabaseHandle;

const COLORS: &[(&str, &str)] = &[
    ("yellow", "bg-yellow-400"),
    ("blue", "bg-blue-400"),
    ("green", "bg-green-400"),
    ("pink", "bg-pink-400"),
];

fn border_class(color: &str) -> &'static str {
    match color {
        "yellow" => "border-l-yellow-400",
        "blue" => "border-l-blue-400",
        "green" => "border-l-green-400",
        "pink" => "border-l-pink-400",
        _ => "border-l-yellow-400",
    }
}

fn selected_ring(current: &str, name: &str) -> &'static str {
    if current == name {
        "ring-2 ring-offset-1 ring-gray-400 dark:ring-gray-300"
    } else {
        ""
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BookNotesProps {
    pub book_id: i64,
}

#[component]
pub fn BookNotes(props: BookNotesProps) -> Element {
    let db = use_context::<DatabaseHandle>();
    let book_id = props.book_id;

    let mut notes: Signal<Vec<BookNote>> = use_signal(Vec::new);
    let mut new_content = use_signal(String::new);
    let mut new_color = use_signal(|| "yellow".to_string());
    let mut editing_id: Signal<Option<i64>> = use_signal(|| None);
    let mut edit_content = use_signal(String::new);
    let mut edit_color = use_signal(String::new);

    {
        let db = db.clone();
        use_effect(move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                if let Ok(n) = list_book_notes_db(&conn, book_id) {
                    notes.set(n);
                }
            });
        });
    }

    let reload_notes = {
        let db = db.clone();
        move || {
            let db = db.clone();
            spawn(async move {
                let conn = db.conn.lock().unwrap();
                if let Ok(n) = list_book_notes_db(&conn, book_id) {
                    notes.set(n);
                }
            });
        }
    };

    rsx! {
        div {
            div {
                class: "mb-2 flex items-center justify-between",
                label {
                    class: "text-xs font-medium text-gray-500 dark:text-gray-400",
                    "Notes"
                }
            }

            // Inline add form
            div {
                class: "mb-3 rounded-lg border border-gray-200 bg-gray-50 p-2 dark:border-gray-700 dark:bg-gray-800/50",
                textarea {
                    class: "w-full resize-none rounded border-0 bg-transparent p-1 text-sm \
                        text-gray-700 placeholder-gray-400 focus:outline-none \
                        dark:text-gray-300 dark:placeholder-gray-500",
                    rows: 2,
                    placeholder: "Add a note...",
                    value: "{new_content}",
                    oninput: move |e| new_content.set(e.value()),
                }
                div {
                    class: "flex items-center justify-between pt-1",
                    div {
                        class: "flex gap-1.5",
                        for &(name, bg) in COLORS {
                            button {
                                r#type: "button",
                                onclick: {
                                    let name = name.to_string();
                                    move |_| new_color.set(name.clone())
                                },
                                class: "h-4 w-4 rounded-full {bg} transition \
                                    {selected_ring(&new_color.read(), name)}",
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        disabled: new_content.read().trim().is_empty(),
                        onclick: {
                            let db = db.clone();
                            let reload = reload_notes.clone();
                            move |_| {
                                let content = new_content.read().trim().to_string();
                                if content.is_empty() {
                                    return;
                                }
                                let color = new_color.read().clone();
                                let db = db.clone();
                                let reload = reload.clone();
                                spawn(async move {
                                    let conn = db.conn.lock().unwrap();
                                    let _ = create_book_note_db(&conn, book_id, &content, &color);
                                    drop(conn);
                                    reload();
                                });
                                new_content.set(String::new());
                            }
                        },
                        class: "rounded bg-amber-600 px-2 py-0.5 text-xs font-medium text-white \
                            hover:bg-amber-700 disabled:opacity-40 disabled:cursor-not-allowed",
                        "Save"
                    }
                }
            }

            // Notes list
            if notes.read().is_empty() {
                div {
                    class: "rounded-lg border border-dashed border-gray-300 px-4 py-4 text-center dark:border-gray-600",
                    p {
                        class: "text-sm text-gray-500 dark:text-gray-400",
                        "No notes yet"
                    }
                    p {
                        class: "mt-1 text-xs text-gray-400 dark:text-gray-500",
                        "Quick annotations, like sticky notes on a book."
                    }
                }
            } else {
                div {
                    class: "space-y-2 max-h-64 overflow-y-auto",
                    for note in notes.read().iter() {
                        {
                            let note_id = note.id;
                            let is_editing = *editing_id.read() == Some(note_id);
                            let pinned = note.pinned;
                            let note_color = note.color.clone();
                            let note_content = note.content.clone();

                            rsx! {
                                div {
                                    key: "{note_id}",
                                    class: "rounded-lg border border-l-4 border-gray-200 bg-gray-50 px-3 py-2 \
                                        dark:border-gray-700 dark:bg-gray-800/50 {border_class(&note_color)}",

                                    if is_editing {
                                        textarea {
                                            class: "w-full resize-none rounded border-0 bg-transparent p-0 text-sm \
                                                text-gray-700 focus:outline-none dark:text-gray-300",
                                            rows: 2,
                                            value: "{edit_content}",
                                            oninput: move |e| edit_content.set(e.value()),
                                        }
                                        div {
                                            class: "flex items-center justify-between pt-1",
                                            div {
                                                class: "flex gap-1.5",
                                                for &(name, bg) in COLORS {
                                                    button {
                                                        r#type: "button",
                                                        onclick: {
                                                            let name = name.to_string();
                                                            move |_| edit_color.set(name.clone())
                                                        },
                                                        class: "h-3.5 w-3.5 rounded-full {bg} transition \
                                                            {selected_ring(&edit_color.read(), name)}",
                                                    }
                                                }
                                            }
                                            div {
                                                class: "flex gap-1",
                                                button {
                                                    r#type: "button",
                                                    onclick: move |_| editing_id.set(None),
                                                    class: "text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300",
                                                    "Cancel"
                                                }
                                                button {
                                                    r#type: "button",
                                                    onclick: {
                                                        let db = db.clone();
                                                        let reload = reload_notes.clone();
                                                        move |_| {
                                                            let content = edit_content.read().trim().to_string();
                                                            if content.is_empty() {
                                                                return;
                                                            }
                                                            let color = edit_color.read().clone();
                                                            let db = db.clone();
                                                            let reload = reload.clone();
                                                            spawn(async move {
                                                                let conn = db.conn.lock().unwrap();
                                                                let _ = update_book_note_db(
                                                                    &conn, note_id, &content, &color, pinned,
                                                                );
                                                                drop(conn);
                                                                reload();
                                                            });
                                                            editing_id.set(None);
                                                        }
                                                    },
                                                    class: "rounded bg-amber-600 px-2 py-0.5 text-xs font-medium text-white hover:bg-amber-700",
                                                    "Save"
                                                }
                                            }
                                        }
                                    } else {
                                        div {
                                            class: "group cursor-pointer",
                                            onclick: {
                                                let nc = note_content.clone();
                                                let ncolor = note_color.clone();
                                                move |_| {
                                                    editing_id.set(Some(note_id));
                                                    edit_content.set(nc.clone());
                                                    edit_color.set(ncolor.clone());
                                                }
                                            },
                                            p {
                                                class: "text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap",
                                                "{note_content}"
                                            }
                                        }
                                        div {
                                            class: "mt-1 flex items-center justify-between",
                                            button {
                                                r#type: "button",
                                                onclick: {
                                                    let db = db.clone();
                                                    let reload = reload_notes.clone();
                                                    let nc = note_content.clone();
                                                    let ncolor = note_color.clone();
                                                    move |_| {
                                                        let db = db.clone();
                                                        let reload = reload.clone();
                                                        let nc = nc.clone();
                                                        let ncolor = ncolor.clone();
                                                        spawn(async move {
                                                            let conn = db.conn.lock().unwrap();
                                                            let _ = update_book_note_db(
                                                                &conn, note_id, &nc, &ncolor, !pinned,
                                                            );
                                                            drop(conn);
                                                            reload();
                                                        });
                                                    }
                                                },
                                                class: if pinned {
                                                    "text-amber-500 hover:text-amber-600"
                                                } else {
                                                    "text-gray-300 hover:text-gray-500 dark:text-gray-600 dark:hover:text-gray-400"
                                                },
                                                title: if pinned { "Unpin" } else { "Pin to top" },
                                                svg {
                                                    class: "h-3.5 w-3.5",
                                                    fill: "currentColor",
                                                    view_box: "0 0 24 24",
                                                    path {
                                                        d: "M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z",
                                                    }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                onclick: {
                                                    let db = db.clone();
                                                    let reload = reload_notes.clone();
                                                    move |_| {
                                                        let db = db.clone();
                                                        let reload = reload.clone();
                                                        spawn(async move {
                                                            let conn = db.conn.lock().unwrap();
                                                            let _ = delete_book_note_db(&conn, note_id);
                                                            drop(conn);
                                                            reload();
                                                        });
                                                    }
                                                },
                                                class: "text-gray-300 hover:text-red-500 dark:text-gray-600 dark:hover:text-red-400",
                                                svg {
                                                    class: "h-3.5 w-3.5",
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
