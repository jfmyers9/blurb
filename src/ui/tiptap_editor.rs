use std::sync::atomic::{AtomicU32, Ordering};

use dioxus::prelude::*;

use super::editor_toolbar::EditorToolbar;

#[component]
pub(crate) fn TipTapEditor(
    content: String,
    on_change: EventHandler<String>,
    editable: bool,
) -> Element {
    let editor_id = use_hook(|| {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("tiptap-editor-{n}")
    });

    // Initialize editor and stream content changes back to Rust via eval channel
    let id = editor_id.clone();
    let init_content = content.clone();
    use_effect(move || {
        let id = id.clone();
        let init_content = init_content.clone();
        spawn(async move {
            let escaped = init_content
                .replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace("${", "\\${");
            let js = format!(
                r#"
                TipTapBridge.init("{id}", `{escaped}`, {editable});
                const editor = window.__tiptapEditors["{id}"];
                if (editor) {{
                    let timer = null;
                    editor.on("update", ({{ editor }}) => {{
                        clearTimeout(timer);
                        timer = setTimeout(() => {{
                            const md = editor.storage.markdown.getMarkdown();
                            dioxus.send(md);
                        }}, 300);
                    }});
                }}
                // Keep eval alive to receive streamed messages
                while (true) {{
                    await new Promise(r => setTimeout(r, 100000));
                }}
                "#,
            );
            let mut eval = document::eval(&js);
            while let Ok(md) = eval.recv::<String>().await {
                on_change.call(md);
            }
        });
    });

    // Sync editable prop
    let id = editor_id.clone();
    use_effect(move || {
        document::eval(&format!(r#"TipTapBridge.setEditable("{id}", {editable})"#,));
    });

    rsx! {
        if editable {
            EditorToolbar { editor_id: editor_id.clone() }
        }
        div { class: "tiptap-editor", id: "{editor_id}" }
    }
}
