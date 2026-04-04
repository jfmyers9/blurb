import { useState } from "react";
import { lookupIsbn, searchCovers } from "../lib/api";
import type { BookMetadata } from "../lib/api";

interface AddBookData {
  title: string;
  author: string;
  isbn: string;
  cover_url?: string | null;
  description?: string | null;
  publisher?: string | null;
  published_date?: string | null;
  page_count?: number | null;
}

interface AddBookFormProps {
  open: boolean;
  onClose: () => void;
  onAdd: (data: AddBookData) => Promise<void>;
}

export default function AddBookForm({ open, onClose, onAdd }: AddBookFormProps) {
  const [title, setTitle] = useState("");
  const [author, setAuthor] = useState("");
  const [isbn, setIsbn] = useState("");
  const [metadata, setMetadata] = useState<BookMetadata | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [lookingUp, setLookingUp] = useState(false);
  const [lookupError, setLookupError] = useState<string | null>(null);
  const [coverUrl, setCoverUrl] = useState("");
  const [coverResults, setCoverResults] = useState<BookMetadata[]>([]);
  const [searchingCovers, setSearchingCovers] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<BookMetadata[]>([]);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [searchDone, setSearchDone] = useState(false);

  if (!open) return null;

  const handleLookup = async () => {
    if (!isbn.trim()) return;
    setLookingUp(true);
    setLookupError(null);
    try {
      const result = await lookupIsbn(isbn.trim());
      setMetadata(result);
      if (result.title && !title.trim()) setTitle(result.title);
      if (result.author && !author.trim()) setAuthor(result.author);
    } catch (e) {
      setLookupError(e instanceof Error ? e.message : String(e));
    } finally {
      setLookingUp(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    setSubmitting(true);
    try {
      await onAdd({
        title: title.trim(),
        author: author.trim(),
        isbn: isbn.trim(),
        cover_url: coverUrl.trim() || metadata?.cover_url,
        description: metadata?.description,
        publisher: metadata?.publisher,
        published_date: metadata?.published_date,
        page_count: metadata?.page_count,
      });
      setTitle("");
      setAuthor("");
      setIsbn("");
      setMetadata(null);
      setLookupError(null);
      setCoverUrl("");
      setCoverResults([]);
      setSearchQuery("");
      setSearchResults([]);
      setSearching(false);
      setSearchError(null);
      setSearchDone(false);
      onClose();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={onClose}
      />
      {/* Modal */}
      <form
        onSubmit={handleSubmit}
        className="relative z-10 w-full max-w-md rounded-xl bg-white p-6
          shadow-xl dark:bg-gray-800"
      >
        <h2 className="mb-4 text-lg font-semibold text-gray-900 dark:text-gray-100">
          Add a Book
        </h2>

        <div className="space-y-3">
          {/* Book search */}
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
              Search books
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => {
                  const val = e.target.value;
                  setSearchQuery(val);
                  if (!val.trim()) {
                    setSearchResults([]);
                    setSearchDone(false);
                    setSearchError(null);
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    if (searchQuery.trim() && !searching) {
                      setSearching(true);
                      setSearchResults([]);
                      setSearchError(null);
                      setSearchDone(false);
                      searchCovers(searchQuery.trim())
                        .then((results) => { setSearchResults(results); setSearchDone(true); })
                        .catch((err) => setSearchError(err instanceof Error ? err.message : String(err)))
                        .finally(() => setSearching(false));
                    }
                  }
                }}
                placeholder="Search by title or author…"
                className="flex-1 rounded-md border border-gray-300 bg-white px-3
                  py-2 text-sm text-gray-900 dark:border-gray-600
                  dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                  focus:ring-amber-500 focus:outline-none"
              />
              <button
                type="button"
                onClick={() => {
                  if (!searchQuery.trim()) return;
                  setSearching(true);
                  setSearchResults([]);
                  setSearchError(null);
                  setSearchDone(false);
                  searchCovers(searchQuery.trim())
                    .then((results) => { setSearchResults(results); setSearchDone(true); })
                    .catch((err) => setSearchError(err instanceof Error ? err.message : String(err)))
                    .finally(() => setSearching(false));
                }}
                disabled={searching || !searchQuery.trim()}
                className="rounded-md bg-gray-100 px-3 py-2 text-sm font-medium
                  text-gray-700 hover:bg-gray-200 disabled:opacity-50
                  disabled:cursor-not-allowed dark:bg-gray-600
                  dark:text-gray-200 dark:hover:bg-gray-500"
              >
                {searching ? "Searching…" : "Search"}
              </button>
            </div>
            {searchResults.length > 0 && (
              <ul className="mt-2 max-h-52 overflow-y-auto rounded-md border
                border-gray-200 dark:border-gray-600">
                {searchResults.map((result, i) => (
                  <li key={i}>
                    <button
                      type="button"
                      onClick={() => {
                        if (result.title) setTitle(result.title);
                        if (result.author) setAuthor(result.author);
                        if (result.isbn) setIsbn(result.isbn);
                        if (result.cover_url) setCoverUrl(result.cover_url);
                        setMetadata(result);
                        setSearchResults([]);
                        setSearchDone(false);
                      }}
                      className="flex w-full items-center gap-3 px-3 py-2 text-left
                        text-sm hover:bg-gray-100 dark:hover:bg-gray-700
                        text-gray-900 dark:text-gray-100"
                    >
                      {result.cover_url ? (
                        <img
                          src={result.cover_url}
                          alt=""
                          className="h-10 w-7 flex-shrink-0 rounded object-cover"
                        />
                      ) : (
                        <div className="h-10 w-7 flex-shrink-0 rounded bg-gray-200
                          dark:bg-gray-600" />
                      )}
                      <div className="min-w-0">
                        <p className="truncate font-medium">
                          {result.title || "Untitled"}
                        </p>
                        {result.author && (
                          <p className="truncate text-xs text-gray-500 dark:text-gray-400">
                            {result.author}
                          </p>
                        )}
                      </div>
                    </button>
                  </li>
                ))}
              </ul>
            )}
            {searchError && (
              <p className="mt-1 text-xs text-red-500">{searchError}</p>
            )}
            {searchDone && searchResults.length === 0 && !searchError && (
              <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">No results found</p>
            )}
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
              ISBN
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={isbn}
                onChange={(e) => setIsbn(e.target.value)}
                placeholder="978-0-14-143951-8"
                className="flex-1 rounded-md border border-gray-300 bg-white px-3
                  py-2 text-sm text-gray-900 dark:border-gray-600
                  dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                  focus:ring-amber-500 focus:outline-none"
              />
              <button
                type="button"
                onClick={handleLookup}
                disabled={lookingUp || !isbn.trim()}
                className="rounded-md bg-gray-100 px-3 py-2 text-sm font-medium
                  text-gray-700 hover:bg-gray-200 disabled:opacity-50
                  disabled:cursor-not-allowed dark:bg-gray-600
                  dark:text-gray-200 dark:hover:bg-gray-500"
              >
                {lookingUp ? "Looking up..." : "Look up"}
              </button>
            </div>
            {lookupError && (
              <p className="mt-1 text-xs text-red-500">{lookupError}</p>
            )}
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
              Cover URL
            </label>
            <input
              type="url"
              value={coverUrl}
              onChange={(e) => setCoverUrl(e.target.value)}
              placeholder="https://example.com/cover.jpg"
              className="w-full rounded-md border border-gray-300 bg-white px-3
                py-2 text-sm text-gray-900 dark:border-gray-600
                dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                focus:ring-amber-500 focus:outline-none"
            />
          </div>

          {(coverUrl.trim() || metadata?.cover_url) && (
            <div className="flex justify-center">
              <img
                src={coverUrl.trim() || metadata?.cover_url || ""}
                alt="Cover preview"
                className="h-32 rounded-md shadow-sm object-contain"
              />
            </div>
          )}

          {title.trim() && (
            <div>
              <button
                type="button"
                onClick={async () => {
                  const query = `${title} ${author}`.trim();
                  setSearchingCovers(true);
                  setCoverResults([]);
                  try {
                    const results = await searchCovers(query);
                    setCoverResults(results);
                  } finally {
                    setSearchingCovers(false);
                  }
                }}
                disabled={searchingCovers}
                className="rounded-md bg-gray-100 px-3 py-2 text-sm font-medium
                  text-gray-700 hover:bg-gray-200 disabled:opacity-50
                  disabled:cursor-not-allowed dark:bg-gray-600
                  dark:text-gray-200 dark:hover:bg-gray-500"
              >
                {searchingCovers ? "Searching..." : "Find cover"}
              </button>
              {coverResults.length > 0 && (
                <div className="mt-2 grid grid-cols-4 gap-2">
                  {coverResults.map((result, i) =>
                    result.cover_url ? (
                      <button
                        key={i}
                        type="button"
                        onClick={() => {
                          setCoverUrl(result.cover_url!);
                          setCoverResults([]);
                        }}
                        className={`overflow-hidden rounded-md border-2 p-0.5
                          hover:border-amber-500 transition-colors ${
                            coverUrl === result.cover_url
                              ? "border-amber-500"
                              : "border-gray-200 dark:border-gray-600"
                          }`}
                      >
                        <img
                          src={result.cover_url}
                          alt={result.title || "Cover option"}
                          className="h-20 w-full object-contain"
                        />
                      </button>
                    ) : null
                  )}
                </div>
              )}
            </div>
          )}

          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
              Title <span className="text-red-500">*</span>
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              autoFocus
              required
              className="w-full rounded-md border border-gray-300 bg-white px-3
                py-2 text-sm text-gray-900 dark:border-gray-600
                dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                focus:ring-amber-500 focus:outline-none"
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
              Author
            </label>
            <input
              type="text"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              className="w-full rounded-md border border-gray-300 bg-white px-3
                py-2 text-sm text-gray-900 dark:border-gray-600
                dark:bg-gray-700 dark:text-gray-100 focus:ring-2
                focus:ring-amber-500 focus:outline-none"
            />
          </div>

          {metadata && (metadata.publisher || metadata.page_count) && (
            <div className="rounded-md bg-gray-50 p-3 text-xs text-gray-600
              dark:bg-gray-700 dark:text-gray-300 space-y-0.5">
              {metadata.publisher && <p>Publisher: {metadata.publisher}</p>}
              {metadata.published_date && <p>Published: {metadata.published_date}</p>}
              {metadata.page_count && <p>Pages: {metadata.page_count}</p>}
            </div>
          )}
        </div>

        <div className="mt-6 flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="rounded-md px-4 py-2 text-sm font-medium text-gray-600
              hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-200"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={submitting || !title.trim()}
            className="rounded-md bg-amber-600 px-4 py-2 text-sm font-medium
              text-white hover:bg-amber-700 disabled:opacity-50
              disabled:cursor-not-allowed"
          >
            {submitting ? "Adding..." : "Add Book"}
          </button>
        </div>
      </form>
    </div>
  );
}
