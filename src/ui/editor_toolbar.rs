use dioxus::prelude::*;

use super::editor_types::{ActiveState, Command};

/// Translate a `Command` into TipTap JS chain call and execute it.
fn dispatch_tiptap(id: &str, cmd: Command) {
    let chain = match cmd {
        Command::ToggleBold => "toggleBold()".to_string(),
        Command::ToggleItalic => "toggleItalic()".to_string(),
        Command::ToggleStrike => "toggleStrike()".to_string(),
        Command::SetHeading(level) => format!("toggleHeading({{level: {level}}})"),
        Command::ToggleBulletList => "toggleBulletList()".to_string(),
        Command::ToggleOrderedList => "toggleOrderedList()".to_string(),
        Command::ToggleBlockquote => "toggleBlockquote()".to_string(),
        Command::ToggleCodeBlock => "toggleCodeBlock()".to_string(),
        Command::InsertHorizontalRule => "setHorizontalRule()".to_string(),
        Command::ToggleLink(url) => {
            let escaped = url.replace('\\', "\\\\").replace('\'', "\\'");
            format!("toggleLink({{ href: '{escaped}' }})")
        }
        Command::Undo => "undo()".to_string(),
        Command::Redo => "redo()".to_string(),
    };
    let js = format!("window.__tiptapEditors['{id}'].chain().focus().{chain}.run()");
    document::eval(&js);
}

/// Poll TipTap active state via JS eval and return an `ActiveState`.
pub(crate) fn poll_tiptap_active_state(id: &str) -> String {
    format!(
        r#"
        const e = window.__tiptapEditors["{id}"];
        if (e) {{
            return JSON.stringify({{
                bold: e.isActive("bold"),
                italic: e.isActive("italic"),
                strike: e.isActive("strike"),
                h1: e.isActive("heading", {{ level: 1 }}),
                h2: e.isActive("heading", {{ level: 2 }}),
                h3: e.isActive("heading", {{ level: 3 }}),
                bulletList: e.isActive("bulletList"),
                orderedList: e.isActive("orderedList"),
                blockquote: e.isActive("blockquote"),
                codeBlock: e.isActive("codeBlock"),
            }});
        }}
        return "null";
        "#,
    )
}

pub(crate) fn parse_active_state(val: &str) -> Option<ActiveState> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Raw {
        bold: bool,
        italic: bool,
        strike: bool,
        h1: bool,
        h2: bool,
        h3: bool,
        bullet_list: bool,
        ordered_list: bool,
        blockquote: bool,
        code_block: bool,
    }
    serde_json::from_str::<Raw>(val)
        .ok()
        .map(|raw| ActiveState {
            bold: raw.bold,
            italic: raw.italic,
            strike: raw.strike,
            h1: raw.h1,
            h2: raw.h2,
            h3: raw.h3,
            bullet_list: raw.bullet_list,
            ordered_list: raw.ordered_list,
            blockquote: raw.blockquote,
            code_block: raw.code_block,
        })
}

#[component]
pub(crate) fn EditorToolbar(editor_id: String) -> Element {
    let mut active = use_signal(ActiveState::default);
    let id_sig = use_signal(|| editor_id.clone());

    let id = editor_id.clone();
    use_future(move || {
        let id = id.clone();
        async move {
            loop {
                let js = poll_tiptap_active_state(&id);
                if let Ok(val) = document::eval(&js).recv::<String>().await {
                    if val != "null" {
                        if let Some(state) = parse_active_state(&val) {
                            active.set(state);
                        }
                    }
                }
                let _ = document::eval("await new Promise(r => setTimeout(r, 250)); return '';")
                    .recv::<String>()
                    .await;
            }
        }
    });

    let btn = "rounded-md p-2 text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800 text-sm";
    let btn_active =
        "rounded-md p-2 text-gray-600 bg-gray-200 dark:text-gray-400 dark:bg-gray-700 text-sm";
    let sep = "w-px h-5 bg-gray-300 dark:bg-gray-600 mx-1";

    let state = active.read();

    rsx! {
        div {
            class: "flex items-center gap-0.5 border-b border-gray-200 px-4 py-2 dark:border-gray-700",

            button {
                class: if state.bold { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::ToggleBold),
                title: "Bold",
                span { class: "font-bold", "B" }
            }
            button {
                class: if state.italic { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::ToggleItalic),
                title: "Italic",
                span { class: "italic", "I" }
            }
            button {
                class: if state.strike { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::ToggleStrike),
                title: "Strikethrough",
                span { class: "line-through", "S" }
            }

            div { class: sep }

            button {
                class: if state.h1 { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::SetHeading(1)),
                title: "Heading 1",
                "H1"
            }
            button {
                class: if state.h2 { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::SetHeading(2)),
                title: "Heading 2",
                "H2"
            }
            button {
                class: if state.h3 { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::SetHeading(3)),
                title: "Heading 3",
                "H3"
            }

            div { class: sep }

            button {
                class: if state.bullet_list { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::ToggleBulletList),
                title: "Bullet List",
                "\u{2022}"
            }
            button {
                class: if state.ordered_list { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::ToggleOrderedList),
                title: "Ordered List",
                "1."
            }
            button {
                class: if state.blockquote { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::ToggleBlockquote),
                title: "Blockquote",
                "\u{201C}"
            }
            button {
                class: if state.code_block { btn_active } else { btn },
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::ToggleCodeBlock),
                title: "Code Block",
                "</>"
            }

            div { class: sep }

            button {
                class: btn,
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::InsertHorizontalRule),
                title: "Horizontal Rule",
                "\u{2015}"
            }
            button {
                class: btn,
                onclick: move |_| {
                    let id = id_sig.read().clone();
                    spawn(async move {
                        let js = "const u = window.prompt('Enter URL:'); return u || '';";
                        if let Ok(url) = document::eval(js).recv::<String>().await {
                            if !url.is_empty() {
                                dispatch_tiptap(&id, Command::ToggleLink(url));
                            }
                        }
                    });
                },
                title: "Link",
                "\u{1F517}"
            }

            div { class: sep }

            button {
                class: btn,
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::Undo),
                title: "Undo",
                "\u{21A9}"
            }
            button {
                class: btn,
                onclick: move |_| dispatch_tiptap(&id_sig.read(), Command::Redo),
                title: "Redo",
                "\u{21AA}"
            }
        }
    }
}
