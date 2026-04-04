import { useState, useEffect, useRef, useCallback } from "react";
import { useEditor, EditorContent } from "@tiptap/react";
import Placeholder from "@tiptap/extension-placeholder";
import { sharedExtensions } from "../lib/editorExtensions";
import {
  createDiaryEntry,
  updateDiaryEntry,
  type DiaryEntry,
} from "../lib/api";
import RatingStars from "./RatingStars";

interface DiaryEntryFormProps {
  bookId: number;
  entry?: DiaryEntry;
  onSave: () => void;
  onClose: () => void;
}

type SaveStatus = "idle" | "saving" | "saved";

function todayString() {
  return new Date().toISOString().slice(0, 10);
}

export default function DiaryEntryForm({
  bookId,
  entry,
  onSave,
  onClose,
}: DiaryEntryFormProps) {
  const [entryDate, setEntryDate] = useState(entry?.entry_date ?? todayString());
  const [rating, setRating] = useState<number | null>(entry?.rating ?? null);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>("idle");
  const [entryId, setEntryId] = useState<number | null>(entry?.id ?? null);

  const dirtyRef = useRef(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const savedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const editorContentRef = useRef<string | null>(entry?.body ?? null);
  const ratingRef = useRef<number | null>(rating);
  const entryDateRef = useRef(entryDate);
  const entryIdRef = useRef<number | null>(entryId);

  ratingRef.current = rating;
  entryDateRef.current = entryDate;
  entryIdRef.current = entryId;

  const doSave = useCallback(
    async (body: string | null) => {
      setSaveStatus("saving");
      try {
        if (entryIdRef.current != null) {
          await updateDiaryEntry(entryIdRef.current, body, ratingRef.current, entryDateRef.current);
        } else {
          const id = await createDiaryEntry(bookId, body, ratingRef.current, entryDateRef.current);
          setEntryId(id);
          entryIdRef.current = id;
        }
        dirtyRef.current = false;
        setSaveStatus("saved");
        onSave();
        savedTimerRef.current = setTimeout(() => setSaveStatus("idle"), 1500);
      } catch {
        setSaveStatus("idle");
      }
    },
    [bookId, onSave]
  );

  const scheduleSave = useCallback(
    (body: string | null) => {
      editorContentRef.current = body;
      dirtyRef.current = true;
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => doSave(body), 2000);
    },
    [doSave]
  );

  // Flush pending save on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
      if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
      if (dirtyRef.current) {
        if (entryIdRef.current != null) {
          updateDiaryEntry(entryIdRef.current, editorContentRef.current, ratingRef.current, entryDateRef.current);
        } else {
          createDiaryEntry(bookId, editorContentRef.current, ratingRef.current, entryDateRef.current);
        }
        onSave();
      }
    };
  }, [bookId, onSave]);

  const initialContent = (() => {
    if (!entry?.body) return undefined;
    try {
      const parsed = JSON.parse(entry.body);
      if (parsed?.type === "doc") return parsed;
    } catch {
      // not JSON
    }
    return undefined;
  })();

  const editor = useEditor(
    {
      extensions: [
        ...sharedExtensions,
        Placeholder.configure({ placeholder: "Write your thoughts..." }),
      ],
      content: initialContent,
      onUpdate: ({ editor: e }) => {
        scheduleSave(JSON.stringify(e.getJSON()));
      },
    },
    []
  );

  const handleRatingChange = (score: number) => {
    const newRating = score === rating ? null : score;
    setRating(newRating);
    ratingRef.current = newRating;
    scheduleSave(editorContentRef.current);
  };

  const handleDateChange = (newDate: string) => {
    setEntryDate(newDate);
    entryDateRef.current = newDate;
    scheduleSave(editorContentRef.current);
  };

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

        <div className="flex min-w-0 flex-1 items-center justify-center gap-3">
          <input
            type="date"
            value={entryDate}
            onChange={(e) => handleDateChange(e.target.value)}
            className="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm
              text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
              focus:ring-2 focus:ring-amber-500 focus:outline-none"
          />
          <RatingStars
            rating={rating}
            onRate={handleRatingChange}
            size="sm"
          />
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
    </div>
  );
}

function Toolbar({ editor }: { editor: ReturnType<typeof useEditor> }) {
  if (!editor) return null;

  const buttons = [
    {
      key: "bold",
      title: "Bold",
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
      title: "Italic",
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
