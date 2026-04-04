import { useState, useCallback, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { generateHTML } from "@tiptap/html";
import StarterKit from "@tiptap/starter-kit";
import type { Book, BookMetadata, Highlight } from "../lib/api";
import { searchCovers, uploadCover, listHighlights, enrichBook } from "../lib/api";
import { coverSrc } from "../lib/cover";
import RatingStars from "./RatingStars";
import StatusSelect from "./StatusSelect";

interface BookDetailProps {
  book: Book;
  onClose: () => void;
  onUpdate: (id: number, title: string, author: string | null) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  onRate: (bookId: number, score: number) => Promise<void>;
  onStatusChange: (bookId: number, status: string) => Promise<void>;
  onEditReview: (bookId: number) => void;
  onLookup: (bookId: number) => Promise<void>;
  onCoverChange: (bookId: number, coverUrl: string) => Promise<void>;
}

export default function BookDetail({
  book,
  onClose,
  onUpdate,
  onDelete,
  onRate,
  onStatusChange,
  onEditReview,
  onLookup,
  onCoverChange,
}: BookDetailProps) {
  const [title, setTitle] = useState(book.title);
  const [author, setAuthor] = useState(book.author ?? "");
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [enriching, setEnriching] = useState(false);

  const [highlights, setHighlights] = useState<Highlight[]>([]);

  useEffect(() => {
    listHighlights(book.id).then(setHighlights).catch(() => setHighlights([]));
  }, [book.id]);

  const [showCoverMenu, setShowCoverMenu] = useState(false);
  const [coverMode, setCoverMode] = useState<"menu" | "paste" | "search" | null>(null);
  const [pasteUrl, setPasteUrl] = useState("");
  const [searchQuery, setSearchQuery] = useState(`${book.title} ${book.author || ""}`.trim());
  const [searchResults, setSearchResults] = useState<BookMetadata[]>([]);
  const [searching, setSearching] = useState(false);

  const handleCoverSearch = useCallback(async (query: string) => {
    if (!query.trim()) return;
    setSearching(true);
    try {
      const results = await searchCovers(query);
      setSearchResults(results);
    } finally {
      setSearching(false);
    }
  }, []);

  const handlePasteSubmit = useCallback(async () => {
    if (!pasteUrl.trim()) return;
    await onCoverChange(book.id, pasteUrl.trim());
    setCoverMode(null);
    setPasteUrl("");
  }, [pasteUrl, book.id, onCoverChange]);

  const handleSelectCover = useCallback(async (url: string) => {
    await onCoverChange(book.id, url);
    setCoverMode(null);
    setSearchResults([]);
  }, [book.id, onCoverChange]);

  const handleTitleBlur = useCallback(() => {
    if (title.trim() && title !== book.title) {
      onUpdate(book.id, title.trim(), author.trim() || null);
    }
  }, [title, author, book.id, book.title, onUpdate]);

  const handleAuthorBlur = useCallback(() => {
    if (author !== (book.author ?? "")) {
      onUpdate(book.id, title.trim() || book.title, author.trim() || null);
    }
  }, [title, author, book.id, book.title, book.author, onUpdate]);

  const handleDelete = async () => {
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    await onDelete(book.id);
    onClose();
  };

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black/30"
        onClick={onClose}
      />
      {/* Panel */}
      <div
        className="fixed top-0 right-0 z-50 flex h-full w-full max-w-md
          flex-col overflow-y-auto bg-white shadow-xl dark:bg-gray-900
          animate-slide-in"
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-gray-200 px-4 py-3 dark:border-gray-700">
          <h2 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            Book Details
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md p-1 text-gray-400 hover:text-gray-600
              dark:hover:text-gray-200"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="flex-1 space-y-5 p-5">
          {/* Cover */}
          <div
            className="group relative mx-auto aspect-[2/3] w-48 overflow-hidden rounded-lg bg-gray-100 dark:bg-gray-700"
            onMouseEnter={() => setShowCoverMenu(true)}
            onMouseLeave={() => { if (!coverMode) setShowCoverMenu(false); }}
          >
            {book.cover_url ? (
              <img
                src={coverSrc(book.cover_url)}
                alt={book.title}
                className="h-full w-full object-cover"
              />
            ) : (
              <div
                className="flex h-full w-full items-center justify-center
                  bg-gradient-to-br from-amber-100 to-orange-200
                  dark:from-amber-900/40 dark:to-orange-900/40"
              >
                <span className="text-6xl font-bold text-amber-700/60 dark:text-amber-400/60">
                  {book.title.charAt(0).toUpperCase()}
                </span>
              </div>
            )}
            {/* Edit overlay */}
            {(showCoverMenu || coverMode) && (
              <div className="absolute inset-0 flex items-end bg-black/40">
                {!coverMode && (
                  <div className="flex w-full flex-col gap-1 p-2">
                    <button
                      type="button"
                      onClick={() => setCoverMode("search")}
                      className="rounded bg-white/90 px-2 py-1 text-xs font-medium text-gray-800 hover:bg-white"
                    >
                      Search cover
                    </button>
                    <button
                      type="button"
                      onClick={() => setCoverMode("paste")}
                      className="rounded bg-white/90 px-2 py-1 text-xs font-medium text-gray-800 hover:bg-white"
                    >
                      Paste URL
                    </button>
                    <button
                      type="button"
                      onClick={async () => {
                        const file = await open({
                          multiple: false,
                          filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "webp"] }],
                        });
                        if (file) {
                          await uploadCover(book.id, file);
                          onCoverChange(book.id, file);
                          setCoverMode(null);
                        }
                      }}
                      className="rounded bg-white/90 px-2 py-1 text-xs font-medium text-gray-800 hover:bg-white"
                    >
                      Upload file
                    </button>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Paste URL input */}
          {coverMode === "paste" && (
            <div className="mx-auto flex w-48 flex-col gap-2">
              <input
                type="url"
                value={pasteUrl}
                onChange={(e) => setPasteUrl(e.target.value)}
                placeholder="https://..."
                autoFocus
                className="w-full rounded-md border border-gray-300 bg-white px-2 py-1.5
                  text-xs text-gray-900 dark:border-gray-600 dark:bg-gray-800
                  dark:text-gray-100 focus:ring-2 focus:ring-amber-500 focus:outline-none"
                onKeyDown={(e) => { if (e.key === "Enter") handlePasteSubmit(); }}
              />
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={handlePasteSubmit}
                  className="flex-1 rounded-md bg-amber-600 px-2 py-1 text-xs font-medium text-white hover:bg-amber-700"
                >
                  Apply
                </button>
                <button
                  type="button"
                  onClick={() => { setCoverMode(null); setPasteUrl(""); }}
                  className="flex-1 rounded-md border border-gray-300 px-2 py-1 text-xs font-medium
                    text-gray-600 hover:bg-gray-50 dark:border-gray-600
                    dark:text-gray-400 dark:hover:bg-gray-800"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {/* Search cover */}
          {coverMode === "search" && (
            <div className="space-y-2">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  autoFocus
                  className="min-w-0 flex-1 rounded-md border border-gray-300 bg-white px-2 py-1.5
                    text-xs text-gray-900 dark:border-gray-600 dark:bg-gray-800
                    dark:text-gray-100 focus:ring-2 focus:ring-amber-500 focus:outline-none"
                  onKeyDown={(e) => { if (e.key === "Enter") handleCoverSearch(searchQuery); }}
                />
                <button
                  type="button"
                  onClick={() => handleCoverSearch(searchQuery)}
                  disabled={searching}
                  className="rounded-md bg-amber-600 px-3 py-1 text-xs font-medium text-white
                    hover:bg-amber-700 disabled:opacity-50"
                >
                  {searching ? "..." : "Search"}
                </button>
                <button
                  type="button"
                  onClick={() => { setCoverMode(null); setSearchResults([]); }}
                  className="rounded-md border border-gray-300 px-2 py-1 text-xs font-medium
                    text-gray-600 hover:bg-gray-50 dark:border-gray-600
                    dark:text-gray-400 dark:hover:bg-gray-800"
                >
                  Cancel
                </button>
              </div>
              {searching && (
                <p className="text-center text-xs text-gray-500">Searching...</p>
              )}
              {searchResults.length > 0 && (
                <div className="grid grid-cols-3 gap-2">
                  {searchResults.map((result, i) => (
                    <button
                      key={i}
                      type="button"
                      onClick={() => result.cover_url && handleSelectCover(result.cover_url)}
                      disabled={!result.cover_url}
                      className="group/thumb flex flex-col gap-1 rounded-md border border-gray-200
                        p-1 text-left hover:border-amber-400 hover:bg-amber-50
                        dark:border-gray-700 dark:hover:border-amber-600
                        dark:hover:bg-amber-900/20 disabled:opacity-40"
                    >
                      {result.cover_url ? (
                        <img
                          src={result.cover_url}
                          alt={result.title ?? ""}
                          className="aspect-[2/3] w-full rounded object-cover"
                        />
                      ) : (
                        <div className="flex aspect-[2/3] w-full items-center justify-center rounded bg-gray-100 dark:bg-gray-700">
                          <span className="text-xs text-gray-400">No img</span>
                        </div>
                      )}
                      <span className="line-clamp-2 text-[10px] leading-tight text-gray-700 dark:text-gray-300">
                        {result.title}
                      </span>
                      {result.author && (
                        <span className="line-clamp-1 text-[10px] leading-tight text-gray-500 dark:text-gray-400">
                          {result.author}
                        </span>
                      )}
                    </button>
                  ))}
                </div>
              )}
              {!searching && searchResults.length === 0 && searchQuery && (
                <p className="text-center text-xs text-gray-400">Press Search to find covers</p>
              )}
            </div>
          )}

          {/* Enrich metadata */}
          {(!book.cover_url || !book.description) && (
            <div className="flex justify-center">
              <button
                type="button"
                disabled={enriching}
                onClick={async () => {
                  setEnriching(true);
                  try {
                    await enrichBook(book.id);
                    await onLookup(book.id);
                  } finally {
                    setEnriching(false);
                  }
                }}
                className="rounded-md bg-amber-600/10 px-3 py-1.5 text-xs font-medium
                  text-amber-600 hover:bg-amber-600/20
                  dark:text-amber-400 dark:hover:bg-amber-600/20
                  disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {enriching ? "Enriching..." : "Enrich metadata"}
              </button>
            </div>
          )}

          {/* Title */}
          <div>
            <label className="mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400">
              Title
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onBlur={handleTitleBlur}
              className="w-full rounded-md border border-gray-300 bg-white px-3
                py-2 text-base font-semibold text-gray-900
                dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
                focus:ring-2 focus:ring-amber-500 focus:outline-none"
            />
          </div>

          {/* Author */}
          <div>
            <label className="mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400">
              Author
            </label>
            <input
              type="text"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              onBlur={handleAuthorBlur}
              placeholder="Unknown"
              className="w-full rounded-md border border-gray-300 bg-white px-3
                py-2 text-sm text-gray-900 dark:border-gray-600
                dark:bg-gray-800 dark:text-gray-100 focus:ring-2
                focus:ring-amber-500 focus:outline-none"
            />
          </div>

          {/* Rating */}
          <div>
            <label className="mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400">
              Rating
            </label>
            <RatingStars
              rating={book.rating}
              onRate={(score) => onRate(book.id, score)}
            />
          </div>

          {/* Status */}
          <div>
            <label className="mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400">
              Status
            </label>
            <StatusSelect
              status={book.status}
              onChange={(status) => onStatusChange(book.id, status)}
            />
          </div>

          {/* Review */}
          <div>
            <div className="mb-1 flex items-center justify-between">
              <label className="text-xs font-medium text-gray-500 dark:text-gray-400">
                Review
              </label>
              <button
                type="button"
                onClick={() => onEditReview(book.id)}
                className="rounded-md px-2 py-1 text-xs font-medium
                  text-amber-600 hover:bg-amber-50
                  dark:text-amber-400 dark:hover:bg-amber-900/20"
              >
                Edit Review
              </button>
            </div>
            {book.review ? (
              (() => {
                try {
                  const doc = JSON.parse(book.review);
                  const html = generateHTML(doc, [StarterKit]);
                  return (
                    <div
                      className="prose prose-sm dark:prose-invert line-clamp-4 text-gray-600 dark:text-gray-400"
                      dangerouslySetInnerHTML={{ __html: html }}
                    />
                  );
                } catch {
                  return (
                    <p className="line-clamp-4 text-sm text-gray-600 dark:text-gray-400">
                      {book.review}
                    </p>
                  );
                }
              })()
            ) : (
              <p className="text-sm text-gray-400 dark:text-gray-500 italic">
                No review yet
              </p>
            )}
          </div>

          {/* Highlights */}
          <div className="border-t border-gray-200 pt-4 dark:border-gray-700">
            <label className="mb-2 block text-xs font-medium text-gray-500 dark:text-gray-400">
              Highlights
            </label>
            {highlights.length === 0 ? (
              <p className="text-sm text-gray-400 dark:text-gray-500 italic">
                No highlights yet
              </p>
            ) : (
              <div className="space-y-3 max-h-64 overflow-y-auto">
                {highlights.map((h) => (
                  <div
                    key={h.id}
                    className="rounded-lg border border-gray-200 bg-gray-50 px-3 py-2
                      dark:border-gray-700 dark:bg-gray-800/50"
                  >
                    <div className="flex items-start gap-2">
                      <svg
                        className="mt-0.5 h-4 w-4 flex-shrink-0 text-amber-500"
                        fill="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path d="M6 17h3l2-4V7H5v6h3zm8 0h3l2-4V7h-6v6h3z" />
                      </svg>
                      {h.text ? (
                        <p className="text-sm text-gray-700 dark:text-gray-300 italic">
                          {h.text}
                        </p>
                      ) : (
                        <p className="text-sm text-gray-400 italic">Bookmark</p>
                      )}
                    </div>
                    <div className="mt-1 flex items-center gap-2 text-[10px] text-gray-500 dark:text-gray-400">
                      <span
                        className={`rounded px-1 py-0.5 font-medium uppercase ${
                          h.clip_type === "highlight"
                            ? "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400"
                            : h.clip_type === "note"
                              ? "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400"
                              : "bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400"
                        }`}
                      >
                        {h.clip_type}
                      </span>
                      {h.location_start != null && (
                        <span>
                          Loc {h.location_start}
                          {h.location_end != null && `-${h.location_end}`}
                        </span>
                      )}
                      {h.page != null && <span>p. {h.page}</span>}
                      {h.clipped_at && (
                        <span>{new Date(h.clipped_at).toLocaleDateString()}</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Metadata */}
          {(book.isbn || book.publisher || book.published_date || book.page_count) && (
            <div className="space-y-1 border-t border-gray-200 pt-4 dark:border-gray-700">
              <div className="mb-2 flex items-center justify-between">
                <h3 className="text-xs font-medium text-gray-500 dark:text-gray-400">
                  Details
                </h3>
                {book.isbn && (
                  <button
                    type="button"
                    disabled={enriching}
                    onClick={async () => {
                      setEnriching(true);
                      try {
                        await onLookup(book.id);
                      } finally {
                        setEnriching(false);
                      }
                    }}
                    className="rounded-md px-2 py-1 text-xs font-medium
                      text-amber-600 hover:bg-amber-50
                      dark:text-amber-400 dark:hover:bg-amber-900/20
                      disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {enriching ? "Looking up..." : "Look up ISBN"}
                  </button>
                )}
              </div>
              {book.isbn && (
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  <span className="font-medium">ISBN:</span> {book.isbn}
                </p>
              )}
              {book.publisher && (
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  <span className="font-medium">Publisher:</span> {book.publisher}
                </p>
              )}
              {book.published_date && (
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  <span className="font-medium">Published:</span> {book.published_date}
                </p>
              )}
              {book.page_count && (
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  <span className="font-medium">Pages:</span> {book.page_count}
                </p>
              )}
            </div>
          )}

          {/* Delete */}
          <div className="border-t border-gray-200 pt-4 dark:border-gray-700">
            <button
              type="button"
              onClick={handleDelete}
              onMouseLeave={() => setConfirmDelete(false)}
              className={`rounded-md px-4 py-2 text-sm font-medium transition ${
                confirmDelete
                  ? "bg-red-600 text-white hover:bg-red-700"
                  : "text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
              }`}
            >
              {confirmDelete ? "Confirm Delete" : "Delete Book"}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
