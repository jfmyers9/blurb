use dioxus::prelude::*;

use crate::hooks::SortOption;

#[derive(Props, Clone, PartialEq)]
pub struct SortDropdownProps {
    value: SortOption,
    on_change: EventHandler<SortOption>,
}

const OPTIONS: &[(SortOption, &str)] = &[
    (SortOption::DateAdded, "Date Added"),
    (SortOption::Title, "Title"),
    (SortOption::Author, "Author"),
    (SortOption::Rating, "Rating"),
];

#[component]
pub fn SortDropdown(props: SortDropdownProps) -> Element {
    let mut open = use_signal(|| false);

    let current_label = OPTIONS
        .iter()
        .find(|(v, _)| *v == props.value)
        .map(|(_, l)| *l)
        .unwrap_or("Sort");

    rsx! {
        div {
            class: "relative",
            button {
                r#type: "button",
                aria_label: "Sort by",
                onclick: move |_| open.toggle(),
                class: "inline-flex items-center gap-1.5 rounded-md border border-gray-300 bg-white
                    px-2.5 py-1.5 text-xs text-gray-700 transition-colors
                    hover:bg-gray-50 focus:ring-2 focus:ring-amber-500 focus:outline-none
                    dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700",
                "{current_label}"
                svg {
                    class: if *open.read() { "h-3 w-3 transition-transform rotate-180" } else { "h-3 w-3 transition-transform" },
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M19 9l-7 7-7-7",
                    }
                }
            }

            if *open.read() {
                ul {
                    role: "listbox",
                    aria_label: "Sort options",
                    class: "absolute right-0 z-20 mt-1 min-w-[10rem] overflow-auto rounded-md border
                        border-gray-300 bg-white py-1 shadow-lg
                        dark:border-gray-600 dark:bg-gray-800",
                    for &(opt_value, label) in OPTIONS.iter() {
                        {
                            let is_selected = opt_value == props.value;
                            let base = if is_selected {
                                "cursor-pointer px-3 py-1.5 text-xs font-medium bg-amber-50 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300"
                            } else {
                                "cursor-pointer px-3 py-1.5 text-xs text-gray-700 dark:text-gray-300 hover:bg-amber-50 hover:text-amber-800 dark:hover:bg-amber-900/30 dark:hover:text-amber-300"
                            };
                            rsx! {
                                li {
                                    key: "{label}",
                                    role: "option",
                                    aria_selected: is_selected,
                                    class: "{base}",
                                    onmousedown: move |evt| {
                                        evt.prevent_default();
                                        props.on_change.call(opt_value);
                                        open.set(false);
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
