import { useState, useEffect, useRef, useCallback } from "react";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { getBook, saveReview, type Book } from "../lib/api";
import { coverSrc } from "../lib/cover";
import { parseReviewContent } from "../lib/reviewParser";

interface ReviewPageProps {
  bookId: number;
  onClose: () => void;
  onSave?: () => void;
}

type SaveStatus = "idle" | "saving" | "saved";

export default function ReviewPage({ bookId, onClose, onSave }: ReviewPageProps) {
  const [book, setBook] = useState<Book | null>(null);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>("idle");
  const dirtyRef = useRef(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const savedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const editorContentRef = useRef<string | null>(null);

  useEffect(() => {
    getBook(bookId).then(setBook);
  }, [bookId]);

  const doSave = useCallback(
    async (json: string) => {
      setSaveStatus("saving");
      try {
        await saveReview(bookId, json);
        dirtyRef.current = false;
        setSaveStatus("saved");
        onSave?.();
        savedTimerRef.current = setTimeout(() => setSaveStatus("idle"), 1500);
      } catch {
        setSaveStatus("idle");
      }
    },
    [bookId, onSave]
  );

  const scheduleSave = useCallback(
    (json: string) => {
      editorContentRef.current = json;
      dirtyRef.current = true;
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => doSave(json), 2000);
    },
    [doSave]
  );

  // Flush pending save on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
      if (dirtyRef.current && editorContentRef.current) {
        saveReview(bookId, editorContentRef.current);
        onSave?.();
      }
    };
  }, [bookId, onSave]);

  const editor = useEditor(
    {
      extensions: [
        StarterKit,
        Placeholder.configure({ placeholder: "Write your thoughts..." }),
      ],
      content: book ? parseReviewContent(book.review) : undefined,
      onUpdate: ({ editor: e }) => {
        scheduleSave(JSON.stringify(e.getJSON()));
      },
    },
    [book]
  );

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-white dark:bg-gray-900">
      {/* Top bar */}
      <div className="flex items-center gap-3 border-b border-gray-200 px-4 py-3 dark:border-gray-700">
        <button
          onClick={onClose}
          className="rounded-md p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200"
          aria-label="Close"
        >
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
          </svg>
        </button>

        <div className="flex min-w-0 flex-1 items-center justify-center gap-2">
          {book?.cover_url && (
            <img
              src={coverSrc(book.cover_url)}
              alt=""
              className="h-8 w-6 rounded-sm object-cover"
            />
          )}
          <span className="truncate text-sm font-medium text-gray-800 dark:text-gray-200">
            {book?.title ?? "Loading..."}
          </span>
        </div>

        <div className="w-16 text-right text-xs text-gray-400">
          {saveStatus === "saving" && "Saving..."}
          {saveStatus === "saved" && (
            <span className="text-green-600 dark:text-green-400">Saved</span>
          )}
        </div>
      </div>

      {/* Toolbar */}
      {editor && <Toolbar editor={editor} />}

      {/* Editor */}
      <div className="flex-1 overflow-y-auto px-4 py-8">
        <div className="mx-auto max-w-3xl">
          <EditorContent
            editor={editor}
            className="prose dark:prose-invert max-w-none
              prose-headings:text-gray-800 dark:prose-headings:text-gray-200
              prose-p:text-gray-700 dark:prose-p:text-gray-300
              focus-within:outline-none min-h-[60vh]"
          />
        </div>
      </div>
    </div>
  );
}

function Toolbar({ editor }: { editor: ReturnType<typeof useEditor> }) {
  if (!editor) return null;

  const buttons = [
    { label: "B", command: () => editor.chain().focus().toggleBold().run(), active: editor.isActive("bold"), style: "font-bold" },
    { label: "I", command: () => editor.chain().focus().toggleItalic().run(), active: editor.isActive("italic"), style: "italic" },
    { label: "H2", command: () => editor.chain().focus().toggleHeading({ level: 2 }).run(), active: editor.isActive("heading", { level: 2 }) },
    { label: "H3", command: () => editor.chain().focus().toggleHeading({ level: 3 }).run(), active: editor.isActive("heading", { level: 3 }) },
    { label: "• List", command: () => editor.chain().focus().toggleBulletList().run(), active: editor.isActive("bulletList") },
    { label: "1. List", command: () => editor.chain().focus().toggleOrderedList().run(), active: editor.isActive("orderedList") },
    { label: "❝", command: () => editor.chain().focus().toggleBlockquote().run(), active: editor.isActive("blockquote") },
  ];

  return (
    <div className="flex items-center gap-1 border-b border-gray-200 px-4 py-1.5 dark:border-gray-700">
      {buttons.map((btn) => (
        <button
          key={btn.label}
          onClick={btn.command}
          className={`rounded px-2 py-1 text-xs transition-colors ${
            btn.active
              ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
              : "text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800"
          } ${btn.style ?? ""}`}
        >
          {btn.label}
        </button>
      ))}
    </div>
  );
}
