import { useState, useEffect, useRef, useCallback } from "react";
import type { Book, DiaryEntry, HighlightSearchResult } from "../lib/api";
import { listDiaryEntries, searchHighlights } from "../lib/api";
import type { Command } from "../lib/commands";

const RECENT_KEY = "blurb-palette-recent";
const MAX_RECENT = 5;

function getRecentKeys(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function addRecentKey(key: string) {
  const keys = getRecentKeys().filter((k) => k !== key);
  keys.unshift(key);
  localStorage.setItem(RECENT_KEY, JSON.stringify(keys.slice(0, MAX_RECENT)));
}

type ResultItem =
  | { type: "command"; data: Command }
  | { type: "book"; data: Book }
  | { type: "diary"; data: DiaryEntry }
  | { type: "highlight"; data: HighlightSearchResult };

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  books: Book[];
  commands: Command[];
  onSelectBook: (book: Book) => void;
}

export default function CommandPalette({
  isOpen,
  onClose,
  books,
  commands,
  onSelectBook,
}: CommandPaletteProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const [query, setQuery] = useState("");
  const [diaryEntries, setDiaryEntries] = useState<DiaryEntry[]>([]);
  const [highlights, setHighlights] = useState<HighlightSearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();

  useEffect(() => {
    if (isOpen) {
      requestAnimationFrame(() => inputRef.current?.focus());
      listDiaryEntries().then((entries) => {
        setDiaryEntries(entries);
      }).catch(() => {});
    } else {
      setQuery("");
      setHighlights([]);
      setSelectedIndex(0);
    }
  }, [isOpen]);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (query.length >= 3) {
      debounceRef.current = setTimeout(() => {
        searchHighlights(query).then(setHighlights).catch(() => setHighlights([]));
      }, 250);
    } else {
      setHighlights([]);
    }
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query]);

  const lowerQ = query.toLowerCase();

  const filteredCommands = query
    ? commands.filter(
        (c) =>
          c.label.toLowerCase().includes(lowerQ) ||
          c.keywords.some((k) => k.toLowerCase().includes(lowerQ))
      )
    : commands;

  const filteredBooks = query
    ? books.filter(
        (b) =>
          b.title.toLowerCase().includes(lowerQ) ||
          (b.author && b.author.toLowerCase().includes(lowerQ))
      )
    : [];

  const filteredDiary = query
    ? diaryEntries.filter(
        (e) =>
          (e.body && e.body.toLowerCase().includes(lowerQ)) ||
          e.book_title.toLowerCase().includes(lowerQ)
      )
    : [];

  // Build recent items when query is empty
  const recentItems: ResultItem[] = [];
  if (!query) {
    const recentKeys = getRecentKeys();
    for (const key of recentKeys) {
      const sepIdx = key.indexOf("-");
      const type = key.slice(0, sepIdx);
      const id = key.slice(sepIdx + 1);
      if (type === "command") {
        const cmd = commands.find((c) => c.id === id);
        if (cmd) recentItems.push({ type: "command", data: cmd });
      } else if (type === "book") {
        const book = books.find((b) => String(b.id) === id);
        if (book) recentItems.push({ type: "book", data: book });
      }
    }
  }

  const sections: { label: string; items: ResultItem[] }[] = [];

  if (!query && recentItems.length > 0) {
    sections.push({ label: "Recent", items: recentItems });
  }

  if (filteredCommands.length > 0)
    sections.push({
      label: "Actions",
      items: filteredCommands.map((c) => ({ type: "command", data: c })),
    });
  if (filteredBooks.length > 0)
    sections.push({
      label: "Books",
      items: filteredBooks.slice(0, 10).map((b) => ({ type: "book", data: b })),
    });
  if (filteredDiary.length > 0)
    sections.push({
      label: "Diary Entries",
      items: filteredDiary.slice(0, 10).map((e) => ({ type: "diary", data: e })),
    });
  if (highlights.length > 0)
    sections.push({
      label: "Highlights",
      items: highlights.slice(0, 10).map((h) => ({ type: "highlight", data: h })),
    });

  const allItems = sections.flatMap((s) => s.items);

  const executeItem = useCallback(
    (item: ResultItem) => {
      addRecentKey(itemKey(item));
      switch (item.type) {
        case "command":
          item.data.execute();
          onClose();
          break;
        case "book":
          onSelectBook(item.data);
          onClose();
          break;
        case "diary": {
          const book = books.find((b) => b.id === item.data.book_id);
          if (book) onSelectBook(book);
          onClose();
          break;
        }
        case "highlight": {
          const hBook = books.find((b) => b.id === item.data.book_id);
          if (hBook) onSelectBook(hBook);
          onClose();
          break;
        }
      }
    },
    [books, onClose, onSelectBook]
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, allItems.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && allItems[selectedIndex]) {
      e.preventDefault();
      executeItem(allItems[selectedIndex]);
    }
  };

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Scroll active item into view
  useEffect(() => {
    const container = listRef.current;
    if (!container) return;
    const active = container.querySelector("[data-active='true']");
    if (active) {
      active.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  if (!isOpen) return null;

  const sectionOffsets = sections.reduce<number[]>((acc, s, i) => {
    acc.push(i === 0 ? 0 : acc[i - 1] + sections[i - 1].items.length);
    return acc;
  }, []);

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="mt-[20vh] w-full max-w-lg rounded-xl border border-gray-200
          bg-white shadow-2xl dark:border-gray-700 dark:bg-gray-900"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center border-b border-gray-200 px-4 dark:border-gray-700">
          <svg
            className="mr-2 h-5 w-5 shrink-0 text-gray-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search books, entries, or actions..."
            className="h-12 w-full bg-transparent text-sm text-gray-900 placeholder-gray-400
              outline-none dark:text-gray-100 dark:placeholder-gray-500"
          />
        </div>
        <div ref={listRef} className="max-h-80 overflow-y-auto p-2">
          {allItems.length === 0 && query ? (
            <div className="flex items-center justify-center py-8 text-sm text-gray-400 dark:text-gray-500">
              No results found
            </div>
          ) : allItems.length === 0 ? (
            <div className="flex items-center justify-center py-8 text-sm text-gray-400 dark:text-gray-500">
              Start typing to search...
            </div>
          ) : (
            sections.map((section, sectionIdx) => (
              <div key={section.label}>
                <div className="px-2 pb-1 pt-2 text-xs font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500">
                  {section.label}
                </div>
                {section.items.map((item, itemIdx) => {
                  const idx = sectionOffsets[sectionIdx] + itemIdx;
                  return (
                    <button
                      key={itemKey(item)}
                      type="button"
                      data-active={idx === selectedIndex}
                      className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition ${
                        idx === selectedIndex
                          ? "bg-amber-50 text-amber-900 dark:bg-amber-900/20 dark:text-amber-100"
                          : "text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800"
                      }`}
                      onClick={() => executeItem(item)}
                      onMouseEnter={() => setSelectedIndex(idx)}
                    >
                      <ResultContent item={item} query={query} />
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

function itemKey(item: { type: string; data: { id: string | number } }) {
  return `${item.type}-${item.data.id}`;
}

function Highlight({ text, query }: { text: string; query: string }) {
  if (!query) return <>{text}</>;
  const lower = text.toLowerCase();
  const idx = lower.indexOf(query.toLowerCase());
  if (idx === -1) return <>{text}</>;
  return (
    <>
      {text.slice(0, idx)}
      <strong className="font-semibold text-amber-600 dark:text-amber-400">
        {text.slice(idx, idx + query.length)}
      </strong>
      {text.slice(idx + query.length)}
    </>
  );
}

function ResultContent({
  item,
  query,
}: {
  item: ResultItem;
  query: string;
}) {
  switch (item.type) {
    case "command":
      return (
        <>
          <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800">
            &gt;
          </span>
          <span><Highlight text={item.data.label} query={query} /></span>
        </>
      );
    case "book":
      return (
        <>
          {item.data.cover_url ? (
            <img
              src={item.data.cover_url}
              alt=""
              className="h-8 w-6 shrink-0 rounded object-cover"
            />
          ) : (
            <span className="flex h-8 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800">
              B
            </span>
          )}
          <div className="min-w-0">
            <div className="truncate font-medium"><Highlight text={item.data.title} query={query} /></div>
            {item.data.author && (
              <div className="truncate text-xs text-gray-400"><Highlight text={item.data.author} query={query} /></div>
            )}
          </div>
        </>
      );
    case "diary":
      return (
        <>
          <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800">
            D
          </span>
          <div className="min-w-0">
            <div className="truncate font-medium">
              <Highlight text={item.data.book_title} query={query} />{" "}
              <span className="font-normal text-gray-400">
                &middot; {item.data.entry_date}
              </span>
            </div>
            {item.data.body && (
              <div className="truncate text-xs text-gray-400">
                <Highlight text={item.data.body.slice(0, 100)} query={query} />
              </div>
            )}
          </div>
        </>
      );
    case "highlight":
      return (
        <>
          <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-gray-100 text-xs dark:bg-gray-800">
            H
          </span>
          <div className="min-w-0">
            <div className="truncate text-xs text-gray-400">
              <Highlight text={item.data.book_title} query={query} />
              {item.data.book_author && <> — <Highlight text={item.data.book_author} query={query} /></>}
            </div>
            <div className="truncate"><Highlight text={item.data.text.slice(0, 120)} query={query} /></div>
          </div>
        </>
      );
  }
}
