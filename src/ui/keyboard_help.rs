use dioxus::prelude::*;

#[component]
pub fn KeyboardHelp(on_close: EventHandler<()>) -> Element {
    let shortcuts = [
        ("\u{2318}K", "Command palette"),
        ("\u{2318}1", "Switch to Library"),
        ("\u{2318}2", "Switch to Diary"),
        ("\u{2318}N", "Add new book"),
        ("\u{2318}I", "Kindle sync"),
        ("/", "Focus search"),
        ("Esc", "Close modals / panels"),
        ("?", "Show this help"),
    ];

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm",
            onclick: move |_| on_close.call(()),

            div {
                class: "w-full max-w-md rounded-xl bg-white p-6 shadow-2xl
                    dark:bg-gray-900 dark:ring-1 dark:ring-gray-700",
                onclick: move |e: MouseEvent| e.stop_propagation(),

                div {
                    class: "mb-4 flex items-center justify-between",
                    h2 {
                        class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                        "Keyboard Shortcuts"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| on_close.call(()),
                        class: "rounded-md p-1 text-gray-400 hover:text-gray-600
                            dark:hover:text-gray-300",
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
                    class: "grid grid-cols-[auto_1fr] gap-x-4 gap-y-2",
                    for (key, desc) in shortcuts {
                        kbd {
                            class: "inline-flex items-center justify-center rounded-md border
                                border-gray-200 bg-gray-50 px-2 py-1 font-mono text-xs
                                font-medium text-gray-600 dark:border-gray-700
                                dark:bg-gray-800 dark:text-gray-300",
                            "{key}"
                        }
                        span {
                            class: "flex items-center text-sm text-gray-700 dark:text-gray-300",
                            "{desc}"
                        }
                    }
                }
            }
        }
    }
}
