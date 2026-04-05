use dioxus::prelude::*;

use crate::data::models::Shelf;

#[derive(Props, Clone, PartialEq)]
pub struct ShelfPickerProps {
    shelves: Vec<Shelf>,
    book_shelf_ids: Vec<i64>,
    on_add: EventHandler<i64>,
    on_remove: EventHandler<i64>,
    on_create: EventHandler<String>,
}

#[component]
pub fn ShelfPicker(props: ShelfPickerProps) -> Element {
    let mut input = use_signal(String::new);
    let mut open = use_signal(|| false);
    let mut creating = use_signal(|| false);

    let book_shelves: Vec<&Shelf> = props
        .shelves
        .iter()
        .filter(|s| props.book_shelf_ids.contains(&s.id))
        .collect();

    let trimmed = input.read().trim().to_lowercase();
    let suggestions: Vec<&Shelf> = props
        .shelves
        .iter()
        .filter(|s| {
            !props.book_shelf_ids.contains(&s.id) && s.name.to_lowercase().contains(&trimmed)
        })
        .collect();
    let exact_match = props
        .shelves
        .iter()
        .any(|s| s.name.to_lowercase() == trimmed);
    let show_create = !trimmed.is_empty() && !exact_match;

    rsx! {
        div {
            // Chips
            if !book_shelves.is_empty() {
                div {
                    class: "mb-2 flex flex-wrap gap-1.5",
                    for shelf in book_shelves.iter() {
                        span {
                            key: "{shelf.id}",
                            class: "inline-flex items-center gap-1 rounded-full bg-amber-100 px-2.5 py-0.5
                                text-xs font-medium text-amber-800 dark:bg-amber-900/40 dark:text-amber-300",
                            "{shelf.name}"
                            button {
                                r#type: "button",
                                onclick: {
                                    let on_remove = props.on_remove;
                                    let id = shelf.id;
                                    move |_| on_remove.call(id)
                                },
                                class: "ml-0.5 rounded-full p-0.5 hover:bg-amber-200 dark:hover:bg-amber-800/40",
                                svg {
                                    class: "h-3 w-3",
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

            // Combo input
            div {
                class: "relative",
                input {
                    r#type: "text",
                    value: "{input}",
                    oninput: move |evt: Event<FormData>| {
                        input.set(evt.value());
                        open.set(true);
                    },
                    onfocus: move |_| open.set(true),
                    onkeydown: {
                        let on_add = props.on_add;
                        let on_create = props.on_create;
                        let suggestions_ids: Vec<i64> = suggestions.iter().map(|s| s.id).collect();
                        move |evt: Event<KeyboardData>| {
                            if evt.key() == Key::Enter {
                                if show_create && suggestions_ids.is_empty() {
                                    let name = input.read().trim().to_string();
                                    if !name.is_empty() && !*creating.read() {
                                        creating.set(true);
                                        on_create.call(name);
                                        input.set(String::new());
                                        open.set(false);
                                        creating.set(false);
                                    }
                                } else if suggestions_ids.len() == 1 {
                                    on_add.call(suggestions_ids[0]);
                                    input.set(String::new());
                                    open.set(false);
                                }
                            }
                        }
                    },
                    placeholder: "Add to shelf...",
                    class: "w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm
                        text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
                        focus:ring-2 focus:ring-amber-500 focus:outline-none",
                }

                if *open.read() && (!suggestions.is_empty() || show_create) {
                    div {
                        class: "absolute z-10 mt-1 w-full rounded-md border border-gray-200 bg-white
                            shadow-lg dark:border-gray-700 dark:bg-gray-800",
                        for shelf in suggestions.iter() {
                            button {
                                key: "{shelf.id}",
                                r#type: "button",
                                onclick: {
                                    let on_add = props.on_add;
                                    let id = shelf.id;
                                    move |_| {
                                        on_add.call(id);
                                        input.set(String::new());
                                        open.set(false);
                                    }
                                },
                                class: "block w-full px-3 py-1.5 text-left text-sm text-gray-900
                                    hover:bg-amber-50 dark:text-gray-100 dark:hover:bg-amber-900/20",
                                "{shelf.name}"
                            }
                        }
                        if show_create {
                            button {
                                r#type: "button",
                                onclick: {
                                    let on_create = props.on_create;
                                    move |_| {
                                        let name = input.read().trim().to_string();
                                        if !name.is_empty() && !*creating.read() {
                                            creating.set(true);
                                            on_create.call(name);
                                            input.set(String::new());
                                            open.set(false);
                                            creating.set(false);
                                        }
                                    }
                                },
                                disabled: *creating.read(),
                                class: "block w-full border-t border-gray-100 px-3 py-1.5 text-left text-sm
                                    font-medium text-amber-600 hover:bg-amber-50
                                    dark:border-gray-700 dark:text-amber-400 dark:hover:bg-amber-900/20
                                    disabled:opacity-50",
                                if *creating.read() {
                                    "Creating..."
                                } else {
                                    "Create \"{input}\""
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
