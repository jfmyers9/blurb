use dioxus::prelude::*;

#[cfg(not(feature = "native-editor"))]
use super::tiptap_editor::TipTapEditor;

/// Wrapper component that delegates to the active editor backend.
/// With `native-editor` feature, shows a stub pointing to the standalone editor binary.
#[component]
pub(crate) fn RichTextEditor(
    content: String,
    on_change: EventHandler<String>,
    editable: bool,
) -> Element {
    #[cfg(feature = "native-editor")]
    {
        let _ = (content, on_change, editable);
        rsx! {
            div {
                class: "flex flex-col items-center justify-center p-8 text-gray-500 dark:text-gray-400 border border-dashed border-gray-300 dark:border-gray-600 rounded-lg m-4",
                p { class: "text-lg font-medium mb-2", "Native Editor" }
                p { class: "text-sm", "The native rich-text editor runs as a standalone window." }
                p { class: "text-xs mt-2 font-mono bg-gray-100 dark:bg-gray-800 px-3 py-1 rounded",
                    "cargo run -p richtext-render --example editor"
                }
            }
        }
    }
    #[cfg(not(feature = "native-editor"))]
    {
        rsx! {
            TipTapEditor { content, on_change, editable }
        }
    }
}
