import { useState, useRef, useEffect } from "react";
import type { Book, Shelf } from "../lib/api";
import { getStatusInfo } from "./StatusSelect";

const FILTER_STATUSES = [
  { value: "all", label: "All" },
  { value: "want_to_read", label: "Want to Read" },
  { value: "reading", label: "Reading" },
  { value: "finished", label: "Finished" },
  { value: "abandoned", label: "Abandoned" },
] as const;

export type FilterStatus = (typeof FILTER_STATUSES)[number]["value"];
export type SortOption = "title" | "author" | "date_added" | "rating";

const SORT_OPTIONS: { value: SortOption; label: string }[] = [
  { value: "date_added", label: "Date Added" },
  { value: "title", label: "Title" },
  { value: "author", label: "Author" },
  { value: "rating", label: "Rating" },
];

interface StatusFilterBarProps {
  books: Book[];
  activeStatus: FilterStatus;
  onStatusChange: (status: FilterStatus) => void;
  sortBy: SortOption;
  onSortChange: (sort: SortOption) => void;
  shelves: Shelf[];
  activeShelf: number | null;
  onShelfChange: (shelfId: number | null) => void;
  shelfBookCounts: Record<number, number>;
  onRenameShelf: (shelfId: number, newName: string) => Promise<void>;
  onDeleteShelf: (shelfId: number, bookCount: number) => Promise<void>;
}

function ShelfPill({
  shelf,
  isActive,
  bookCount,
  onClick,
  onRename,
  onDelete,
}: {
  shelf: Shelf;
  isActive: boolean;
  bookCount: number;
  onClick: () => void;
  onRename: (newName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState(shelf.name);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editing) inputRef.current?.select();
  }, [editing]);

  const commitRename = async () => {
    const trimmed = editName.trim();
    if (trimmed && trimmed !== shelf.name) {
      try {
        await onRename(trimmed);
      } catch {
        setEditName(shelf.name);
        return;
      }
    } else {
      setEditName(shelf.name);
    }
    setEditing(false);
  };

  if (editing) {
    return (
      <input
        ref={inputRef}
        value={editName}
        onChange={(e) => setEditName(e.target.value)}
        onBlur={commitRename}
        onKeyDown={(e) => {
          if (e.key === "Enter") commitRename();
          if (e.key === "Escape") {
            setEditName(shelf.name);
            setEditing(false);
          }
        }}
        className="rounded-full border border-amber-400 bg-white px-3 py-1.5 text-xs font-medium
          text-gray-800 outline-none dark:border-amber-600 dark:bg-gray-800 dark:text-gray-200"
      />
    );
  }

  return (
    <span className="group relative inline-flex items-center">
      <button
        type="button"
        onClick={onClick}
        className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium
          transition-colors ${
            isActive
              ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
              : "bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
          }`}
      >
        {shelf.name}
        <span
          className={`inline-flex h-4.5 min-w-[1.125rem] items-center justify-center rounded-full px-1 text-[10px] font-semibold leading-none ${
            isActive
              ? "bg-white/40 dark:bg-black/20"
              : "bg-gray-200/80 dark:bg-gray-700"
          }`}
        >
          {bookCount}
        </span>
      </button>
      {/* Edit/delete icons on hover */}
      <span className="ml-0.5 hidden items-center gap-0.5 group-hover:inline-flex">
        <button
          type="button"
          title="Rename shelf"
          onClick={(e) => {
            e.stopPropagation();
            setEditName(shelf.name);
            setEditing(true);
          }}
          className="flex h-5 w-5 items-center justify-center rounded-full text-gray-400
            hover:bg-gray-200 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
        >
          <svg className="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
              d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
        </button>
        <button
          type="button"
          title="Delete shelf"
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          className="flex h-5 w-5 items-center justify-center rounded-full text-gray-400
            hover:bg-red-100 hover:text-red-600 dark:hover:bg-red-900/30 dark:hover:text-red-400"
        >
          <svg className="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </span>
    </span>
  );
}

export default function StatusFilterBar({
  books,
  activeStatus,
  onStatusChange,
  sortBy,
  onSortChange,
  shelves,
  activeShelf,
  onShelfChange,
  shelfBookCounts,
  onRenameShelf,
  onDeleteShelf,
}: StatusFilterBarProps) {
  const counts = new Map<string, number>();
  counts.set("all", books.length);
  for (const book of books) {
    const s = book.status ?? "";
    if (s) counts.set(s, (counts.get(s) ?? 0) + 1);
  }

  return (
    <div className="space-y-2 px-6 pt-5 pb-1">
      {/* Status row */}
      <div className="flex items-center justify-between gap-4">
        <div className="flex flex-wrap gap-1.5">
          {FILTER_STATUSES.map((tab) => {
            const isActive = activeStatus === tab.value;
            const count = counts.get(tab.value) ?? 0;
            const statusInfo =
              tab.value !== "all" ? getStatusInfo(tab.value) : null;

            return (
              <button
                key={tab.value}
                type="button"
                onClick={() => onStatusChange(tab.value)}
                className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium
                  transition-colors ${
                    isActive
                      ? statusInfo?.color ||
                        "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                      : "bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                  }`}
              >
                {tab.label}
                <span
                  className={`inline-flex h-4.5 min-w-[1.125rem] items-center justify-center rounded-full px-1 text-[10px] font-semibold leading-none ${
                    isActive
                      ? "bg-white/40 dark:bg-black/20"
                      : "bg-gray-200/80 dark:bg-gray-700"
                  }`}
                >
                  {count}
                </span>
              </button>
            );
          })}
        </div>

        <select
          value={sortBy}
          onChange={(e) => onSortChange(e.target.value as SortOption)}
          className="rounded-md border border-gray-300 bg-white px-2.5 py-1.5 text-xs
            text-gray-700 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300
            focus:ring-2 focus:ring-amber-500 focus:outline-none"
        >
          {SORT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      {/* Shelf row */}
      {shelves.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5">
          <button
            type="button"
            onClick={() => onShelfChange(null)}
            className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium
              transition-colors ${
                activeShelf === null
                  ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                  : "bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
              }`}
          >
            All Shelves
          </button>
          {shelves.map((shelf) => (
            <ShelfPill
              key={shelf.id}
              shelf={shelf}
              isActive={activeShelf === shelf.id}
              bookCount={shelfBookCounts[shelf.id] ?? 0}
              onClick={() => onShelfChange(shelf.id)}
              onRename={(newName) => onRenameShelf(shelf.id, newName)}
              onDelete={() => onDeleteShelf(shelf.id, shelfBookCounts[shelf.id] ?? 0)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
