import { useState, useEffect, useRef, useCallback } from "react";
import { useEditor, EditorContent } from "@tiptap/react";
import Placeholder from "@tiptap/extension-placeholder";
import { sharedExtensions } from "../lib/editorExtensions";
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
    const backup = localStorage.getItem(`review-backup-${bookId}`);
    if (backup) {
      saveReview(bookId, backup).then(() => {
        localStorage.removeItem(`review-backup-${bookId}`);
      });
    }
    getBook(bookId).then(setBook);
  }, [bookId]);

  const doSave = useCallback(
    async (json: string) => {
      setSaveStatus("saving");
      try {
        await saveReview(bookId, json);
        dirtyRef.current = false;
        localStorage.removeItem(`review-backup-${bookId}`);
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
        localStorage.setItem(`review-backup-${bookId}`, editorContentRef.current);
        saveReview(bookId, editorContentRef.current);
        onSave?.();
      }
    };
  }, [bookId, onSave]);

  const editor = useEditor(
    {
      extensions: [
        ...sharedExtensions,
        Placeholder.configure({ placeholder: "Write your thoughts..." }),
      ],
      content: book ? parseReviewContent(book.review) : undefined,
      onUpdate: ({ editor: e }) => {
        scheduleSave(JSON.stringify(e.getJSON()));
      },
    },
    [book]
  );

  const wordCount = editor?.getText().split(/\s+/).filter(Boolean).length ?? 0;

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
      <div className="flex-1 overflow-y-auto px-6 py-12 bg-gray-50 dark:bg-gray-900">
        <div className="mx-auto max-w-3xl bg-white dark:bg-gray-800/80 shadow-sm rounded-xl px-10 py-12 ring-1 ring-gray-200/50 dark:ring-gray-700/50 min-h-[60vh]">
          <EditorContent
            editor={editor}
            className="prose dark:prose-invert max-w-none
              prose-headings:text-gray-800 dark:prose-headings:text-gray-200
              prose-p:text-gray-700 dark:prose-p:text-gray-300
              focus-within:outline-none"
          />
        </div>
      </div>

      {/* Word count */}
      <div className="flex items-center justify-end px-6 py-2 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-400">
        {`${wordCount} ${wordCount === 1 ? "word" : "words"}`}
      </div>
    </div>
  );
}

function Toolbar({ editor }: { editor: ReturnType<typeof useEditor> }) {
  if (!editor) return null;

  const buttons = [
    {
      key: "bold",
      title: "Bold (⌘B)",
      command: () => editor.chain().focus().toggleBold().run(),
      active: editor.isActive("bold"),
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5} strokeLinecap="round" strokeLinejoin="round">
          <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
          <path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
        </svg>
      ),
    },
    {
      key: "italic",
      title: "Italic (⌘I)",
      command: () => editor.chain().focus().toggleItalic().run(),
      active: editor.isActive("italic"),
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
          <line x1="19" y1="4" x2="10" y2="4" />
          <line x1="14" y1="20" x2="5" y2="20" />
          <line x1="15" y1="4" x2="9" y2="20" />
        </svg>
      ),
    },
    {
      key: "h2",
      title: "Heading 2",
      command: () => editor.chain().focus().toggleHeading({ level: 2 }).run(),
      active: editor.isActive("heading", { level: 2 }),
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
          <path d="M4 12h8" />
          <path d="M4 18V6" />
          <path d="M12 18V6" />
          <path d="M17 12a2 2 0 1 1 4 0c0 1-1.5 2-4 4h4" />
        </svg>
      ),
    },
    {
      key: "h3",
      title: "Heading 3",
      command: () => editor.chain().focus().toggleHeading({ level: 3 }).run(),
      active: editor.isActive("heading", { level: 3 }),
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
          <path d="M4 12h8" />
          <path d="M4 18V6" />
          <path d="M12 18V6" />
          <path d="M17.5 10.5c1.7-1 3.5 0 3.5 1.5a2 2 0 0 1-2 2 2 2 0 0 1 2 2c0 1.5-1.8 2.5-3.5 1.5" />
        </svg>
      ),
    },
    {
      key: "bulletList",
      title: "Bullet List",
      command: () => editor.chain().focus().toggleBulletList().run(),
      active: editor.isActive("bulletList"),
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
          <line x1="9" y1="6" x2="20" y2="6" />
          <line x1="9" y1="12" x2="20" y2="12" />
          <line x1="9" y1="18" x2="20" y2="18" />
          <circle cx="5" cy="6" r="1" fill="currentColor" />
          <circle cx="5" cy="12" r="1" fill="currentColor" />
          <circle cx="5" cy="18" r="1" fill="currentColor" />
        </svg>
      ),
    },
    {
      key: "orderedList",
      title: "Ordered List",
      command: () => editor.chain().focus().toggleOrderedList().run(),
      active: editor.isActive("orderedList"),
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
          <line x1="10" y1="6" x2="20" y2="6" />
          <line x1="10" y1="12" x2="20" y2="12" />
          <line x1="10" y1="18" x2="20" y2="18" />
          <text x="3" y="8" fontSize="7" fill="currentColor" stroke="none" fontFamily="sans-serif">1</text>
          <text x="3" y="14" fontSize="7" fill="currentColor" stroke="none" fontFamily="sans-serif">2</text>
          <text x="3" y="20" fontSize="7" fill="currentColor" stroke="none" fontFamily="sans-serif">3</text>
        </svg>
      ),
    },
    {
      key: "blockquote",
      title: "Blockquote",
      command: () => editor.chain().focus().toggleBlockquote().run(),
      active: editor.isActive("blockquote"),
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
          <path d="M3 21c3-3 4-6 4-9s-1-4-4-4" />
          <path d="M14 21c3-3 4-6 4-9s-1-4-4-4" />
        </svg>
      ),
    },
  ];

  return (
    <div className="flex items-center gap-0.5 border-b border-gray-200 px-4 py-2 dark:border-gray-700">
      {buttons.map((btn) => (
        <button
          key={btn.key}
          title={btn.title}
          onClick={btn.command}
          className={`p-2 rounded-md transition-colors ${
            btn.active
              ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
              : "text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800"
          }`}
        >
          {btn.icon}
        </button>
      ))}
    </div>
  );
}
