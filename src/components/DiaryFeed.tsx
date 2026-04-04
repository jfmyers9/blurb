import { useState, useEffect, useCallback } from "react";
import { generateHTML } from "@tiptap/html";
import { sharedExtensions } from "../lib/editorExtensions";
import { listDiaryEntries } from "../lib/api";
import type { DiaryEntry } from "../lib/api";
import { coverSrc } from "../lib/cover";
import RatingStars from "./RatingStars";

interface DiaryFeedProps {
  onSelectBook: (bookId: number) => void;
}

function extractPlainText(body: string): string {
  try {
    const doc = JSON.parse(body);
    const html = generateHTML(doc, sharedExtensions);
    const tmp = document.createElement("div");
    tmp.innerHTML = html;
    const text = tmp.textContent ?? "";
    return text.length > 150 ? text.slice(0, 150) + "…" : text;
  } catch {
    return body.length > 150 ? body.slice(0, 150) + "…" : body;
  }
}

function formatDate(dateStr: string): string {
  const [year, month, day] = dateStr.split("-").map(Number);
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

function monthKey(dateStr: string): string {
  const [year, month] = dateStr.split("-").map(Number);
  const date = new Date(year, month - 1);
  return date.toLocaleDateString("en-US", { year: "numeric", month: "long" });
}

export default function DiaryFeed({ onSelectBook }: DiaryFeedProps) {
  const [entries, setEntries] = useState<DiaryEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const data = await listDiaryEntries();
      setEntries(data);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  if (loading && entries.length === 0) {
    return (
      <div className="flex items-center justify-center py-20">
        <p className="text-sm text-gray-400 dark:text-gray-500">Loading diary…</p>
      </div>
    );
  }

  if (entries.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <p className="text-sm text-gray-400 dark:text-gray-500">No diary entries yet</p>
        <p className="mt-1 text-xs text-gray-400 dark:text-gray-600">
          Add entries from a book's detail page
        </p>
      </div>
    );
  }

  const grouped: { month: string; entries: DiaryEntry[] }[] = [];
  let currentMonth = "";
  for (const entry of entries) {
    const mk = monthKey(entry.entry_date);
    if (mk !== currentMonth) {
      currentMonth = mk;
      grouped.push({ month: mk, entries: [] });
    }
    grouped[grouped.length - 1].entries.push(entry);
  }

  return (
    <div className="mx-auto max-w-2xl px-4 py-6">
      {grouped.map((group) => (
        <div key={group.month} className="mb-8">
          <h2 className="mb-3 text-sm font-semibold text-gray-500 dark:text-gray-400">
            {group.month}
          </h2>
          <div className="space-y-3">
            {group.entries.map((entry) => (
              <button
                key={entry.id}
                type="button"
                onClick={() => onSelectBook(entry.book_id)}
                className="flex w-full gap-3 rounded-lg border border-gray-200 bg-white
                  p-3 text-left transition hover:border-amber-300 hover:shadow-sm
                  dark:border-gray-700 dark:bg-gray-900 dark:hover:border-amber-600"
              >
                {/* Cover thumbnail */}
                <div className="h-16 w-11 flex-shrink-0 overflow-hidden rounded bg-gray-100 dark:bg-gray-700">
                  {entry.book_cover_url ? (
                    <img
                      src={coverSrc(entry.book_cover_url)}
                      alt={entry.book_title}
                      className="h-full w-full object-cover"
                    />
                  ) : (
                    <div
                      className="flex h-full w-full items-center justify-center
                        bg-gradient-to-br from-amber-100 to-orange-200
                        dark:from-amber-900/40 dark:to-orange-900/40"
                    >
                      <span className="text-sm font-bold text-amber-700/60 dark:text-amber-400/60">
                        {entry.book_title.charAt(0).toUpperCase()}
                      </span>
                    </div>
                  )}
                </div>

                {/* Content */}
                <div className="min-w-0 flex-1">
                  <div className="flex items-start justify-between gap-2">
                    <div className="min-w-0">
                      <p className="truncate text-sm font-medium text-gray-900 dark:text-gray-100">
                        {entry.book_title}
                      </p>
                      {entry.book_author && (
                        <p className="truncate text-xs text-gray-500 dark:text-gray-400">
                          {entry.book_author}
                        </p>
                      )}
                    </div>
                    <span className="flex-shrink-0 text-xs text-gray-400 dark:text-gray-500">
                      {formatDate(entry.entry_date)}
                    </span>
                  </div>

                  {entry.rating && (
                    <div className="mt-1">
                      <RatingStars rating={entry.rating} onRate={() => {}} size="sm" />
                    </div>
                  )}

                  {entry.body && (
                    <p className="mt-1 text-xs leading-relaxed text-gray-600 dark:text-gray-400 line-clamp-2">
                      {extractPlainText(entry.body)}
                    </p>
                  )}
                </div>
              </button>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
