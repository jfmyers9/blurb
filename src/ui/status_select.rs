use dioxus::prelude::*;

struct StatusOption {
    value: &'static str,
    label: &'static str,
}

const STATUS_OPTIONS: &[StatusOption] = &[
    StatusOption {
        value: "",
        label: "No status",
    },
    StatusOption {
        value: "want_to_read",
        label: "Want to Read",
    },
    StatusOption {
        value: "reading",
        label: "Reading",
    },
    StatusOption {
        value: "finished",
        label: "Finished",
    },
    StatusOption {
        value: "abandoned",
        label: "Abandoned",
    },
];

#[derive(Props, Clone, PartialEq)]
pub struct StatusSelectProps {
    status: Option<String>,
    on_change: EventHandler<String>,
}

#[component]
pub fn StatusSelect(props: StatusSelectProps) -> Element {
    let current = props.status.as_deref().unwrap_or("");

    rsx! {
        select {
            value: "{current}",
            onchange: move |evt: Event<FormData>| {
                props.on_change.call(evt.value());
            },
            class: "rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm
                text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
                focus:ring-2 focus:ring-amber-500 focus:outline-none",
            for opt in STATUS_OPTIONS.iter() {
                option {
                    value: "{opt.value}",
                    selected: current == opt.value,
                    "{opt.label}"
                }
            }
        }
    }
}
