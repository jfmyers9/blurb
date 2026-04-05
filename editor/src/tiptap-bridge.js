import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { Markdown } from "tiptap-markdown";

window.__tiptapEditors = {};

window.TipTapBridge = {
  init(elementId, markdown, editable) {
    // Destroy existing instance if any
    if (window.__tiptapEditors[elementId]) {
      window.__tiptapEditors[elementId].destroy();
    }

    const element = document.getElementById(elementId);
    if (!element) {
      console.error(`TipTapBridge: element #${elementId} not found`);
      return;
    }

    const editor = new Editor({
      element,
      extensions: [
        StarterKit.configure({
          heading: { levels: [1, 2, 3] },
        }),
        Markdown.configure({
          html: false,
          transformCopiedText: true,
          transformPastedText: true,
        }),
      ],
      content: markdown,
      editable,
    });

    window.__tiptapEditors[elementId] = editor;

    if (editable) {
      // Defer focus so the DOM is ready
      setTimeout(() => editor.commands.focus(), 50);
    }
  },

  getMarkdown(elementId) {
    const editor = window.__tiptapEditors[elementId];
    if (!editor) return "";
    return editor.storage.markdown.getMarkdown();
  },

  setMarkdown(elementId, md) {
    const editor = window.__tiptapEditors[elementId];
    if (!editor) return;
    editor.commands.setContent(md);
  },

  setEditable(elementId, editable) {
    const editor = window.__tiptapEditors[elementId];
    if (!editor) return;
    editor.setEditable(editable);
    if (editable) {
      setTimeout(() => editor.commands.focus(), 50);
    }
  },

  destroy(elementId) {
    const editor = window.__tiptapEditors[elementId];
    if (!editor) return;
    editor.destroy();
    delete window.__tiptapEditors[elementId];
  },
};
