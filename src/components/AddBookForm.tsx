import { useState } from "react";
import { lookupIsbn } from "../lib/api";
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
        cover_url: metadata?.cover_url,
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

          {metadata?.cover_url && (
            <div className="flex justify-center">
              <img
                src={metadata.cover_url}
                alt="Cover preview"
                className="h-32 rounded-md shadow-sm object-contain"
              />
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
