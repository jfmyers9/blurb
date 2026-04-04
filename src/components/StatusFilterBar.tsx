import { useState, useRef, useEffect, useMemo } from "react";
import type { Book, Shelf } from "../lib/api";
import { getStatusInfo } from "./StatusSelect";
import SortDropdown from "./ui/SortDropdown";

const FILTER_STATUSES = [
  { value: "all", label: "All" },
  { value: "want_to_read", label: "Want to Read" },
  { value: "reading", label: "Reading" },
  { value: "finished", label: "Finished" },
  { value: "abandoned", label: "Abandoned" },
] as const;

export type FilterStatus = (typeof FILTER_STATUSES)[number]["value"];
export type SortOption = "title" | "author" | "date_added" | "rating";
export type ViewMode = "grid" | "list";

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
  searchQuery: string;
  onSearchChange: (q: string) => void;
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
  minRating: number | null;
  onMinRatingChange: (rating: number | null) => void;
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  onClearAll: () => void;
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
            e.stopPropagation();
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
          active:scale-95 transition-all duration-150 ${
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

function FilterTag({ label, testId, onDismiss }: { label: React.ReactNode; testId: string; onDismiss: () => void }) {
  return (
    <span className="inline-flex items-center gap-1 rounded-full bg-amber-50 px-2.5 py-1 text-xs font-medium text-amber-800 dark:bg-amber-900/30 dark:text-amber-300">
      {label}
      <button
        type="button"
        data-testid={testId}
        onClick={onDismiss}
        className="ml-0.5 text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200"
      >
        ×
      </button>
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
  searchQuery,
  onSearchChange,
  viewMode,
  onViewModeChange,
  minRating,
  onMinRatingChange,
  searchInputRef,
  onClearAll,
}: StatusFilterBarProps) {
  const [isStuck, setIsStuck] = useState(false);
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel) return;
    const observer = new IntersectionObserver(
      ([entry]) => setIsStuck(!entry.isIntersecting),
      { threshold: 0, rootMargin: "-49px 0px 0px 0px" } // Must match header height in App.tsx
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, []);

  const counts = useMemo(() => {
    const m = new Map<string, number>();
    m.set("all", books.length);
    for (const book of books) {
      const s = book.status ?? "";
      if (s) m.set(s, (m.get(s) ?? 0) + 1);
    }
    return m;
  }, [books]);

  return (
    <>
    <div ref={sentinelRef} className="h-0" />
    {/* top-[49px] must match header height in App.tsx */}
    <div className={`sticky top-[49px] z-20 space-y-2 px-6 pt-5 pb-1 bg-white/80 backdrop-blur dark:bg-gray-900/80 transition-shadow duration-150 ${
      isStuck ? "shadow-sm border-b border-gray-200 dark:border-gray-700" : ""
    }`}>
      {/* Status row */}
      <div className="flex items-center justify-between gap-4">
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="text-[10px] font-medium uppercase tracking-wider text-gray-400">Status</span>
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
                  active:scale-95 transition-all duration-150 ${
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

        <div className="relative">
          <svg
            className="pointer-events-none absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-gray-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
            />
          </svg>
          <input
            ref={searchInputRef}
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search..."
            className="w-48 focus:w-64 transition-all duration-200 rounded-md border border-gray-300 bg-white py-1.5 pl-7 pr-2.5 text-xs
              text-gray-700 placeholder-gray-400 dark:border-gray-600 dark:bg-gray-800
              dark:text-gray-300 dark:placeholder-gray-500
              focus:ring-2 focus:ring-amber-500 focus:outline-none"
          />
        </div>

        <SortDropdown value={sortBy} onChange={onSortChange} options={SORT_OPTIONS} />

        <div className="flex gap-0.5">
          <button
            type="button"
            title="Grid view"
            onClick={() => onViewModeChange("grid")}
            className={`flex h-8 w-8 items-center justify-center rounded-md transition-colors ${
              viewMode === "grid"
                ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                : "text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
            }`}
          >
            <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 16 16">
              <rect x="1" y="1" width="6" height="6" rx="1" />
              <rect x="9" y="1" width="6" height="6" rx="1" />
              <rect x="1" y="9" width="6" height="6" rx="1" />
              <rect x="9" y="9" width="6" height="6" rx="1" />
            </svg>
          </button>
          <button
            type="button"
            title="List view"
            onClick={() => onViewModeChange("list")}
            className={`flex h-8 w-8 items-center justify-center rounded-md transition-colors ${
              viewMode === "list"
                ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                : "text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
            }`}
          >
            <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 16 16">
              <rect x="1" y="2" width="14" height="2" rx="0.5" />
              <rect x="1" y="7" width="14" height="2" rx="0.5" />
              <rect x="1" y="12" width="14" height="2" rx="0.5" />
            </svg>
          </button>
        </div>
      </div>

      {/* Rating row */}
      {books.some((b) => b.rating != null) && (
        <div className="flex flex-wrap items-center gap-1.5 border-t border-gray-200 pt-2 dark:border-gray-700">
          <span className="text-[10px] font-medium uppercase tracking-wider text-gray-400">Rating</span>
          {([
            { value: null, label: "Any Rating" },
            { value: 3, label: "3+" },
            { value: 4, label: "4+" },
            { value: 5, label: "5" },
          ] as const).map((opt) => {
            const isActive = minRating === opt.value;
            return (
              <button
                key={opt.label}
                type="button"
                onClick={() => onMinRatingChange(opt.value)}
                className={`rounded-full px-3 py-1.5 text-xs font-medium active:scale-95 transition-all duration-150 ${
                  isActive
                    ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                    : "bg-gray-100 text-gray-600 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
                }`}
              >
                {opt.value !== null && (
                  <span className="text-[10px] opacity-60">★</span>
                )}
                {opt.label}
              </button>
            );
          })}
        </div>
      )}

      {/* Shelf row */}
      {shelves.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5 border-t border-gray-200 pt-2 dark:border-gray-700">
          <span className="text-[10px] font-medium uppercase tracking-wider text-gray-400">Shelves</span>
          <button
            type="button"
            onClick={() => onShelfChange(null)}
            className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium
              active:scale-95 transition-all duration-150 ${
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

      {/* Active filter summary strip */}
      {(activeStatus !== "all" || minRating !== null || activeShelf !== null || searchQuery !== "") && (
        <div
          data-testid="filter-summary"
          className="flex flex-wrap items-center gap-1.5 border-t border-gray-200 pt-2 dark:border-gray-700"
        >
          <span className="text-[10px] font-medium uppercase tracking-wider text-gray-400">Filters</span>
          {activeStatus !== "all" && (
            <FilterTag label={FILTER_STATUSES.find((s) => s.value === activeStatus)?.label} testId="dismiss-status" onDismiss={() => onStatusChange("all")} />
          )}
          {minRating !== null && (
            <FilterTag label={<>{minRating}+ ★</>} testId="dismiss-rating" onDismiss={() => onMinRatingChange(null)} />
          )}
          {activeShelf !== null && (
            <FilterTag label={shelves.find((s) => s.id === activeShelf)?.name} testId="dismiss-shelf" onDismiss={() => onShelfChange(null)} />
          )}
          {searchQuery !== "" && (
            <FilterTag label={<>search: {searchQuery}</>} testId="dismiss-search" onDismiss={() => onSearchChange("")} />
          )}
          <button
            type="button"
            onClick={onClearAll}
            className="text-xs font-medium text-amber-700 hover:text-amber-900 dark:text-amber-400 dark:hover:text-amber-200"
          >
            Clear all
          </button>
        </div>
      )}
    </div>
    </>
  );
}
