import { useState, useCallback } from "react";
import type { Book } from "../lib/api";
import RatingStars from "./RatingStars";
import StatusSelect from "./StatusSelect";
import ReviewEditor from "./ReviewEditor";

interface BookDetailProps {
  book: Book;
  onClose: () => void;
  onUpdate: (id: number, title: string, author: string | null) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
  onRate: (bookId: number, score: number) => Promise<void>;
  onStatusChange: (bookId: number, status: string) => Promise<void>;
  onReviewSave: (bookId: number, body: string) => Promise<void>;
  onLookup: (bookId: number) => Promise<void>;
}

export default function BookDetail({
  book,
  onClose,
  onUpdate,
  onDelete,
  onRate,
  onStatusChange,
  onReviewSave,
  onLookup,
}: BookDetailProps) {
  const [title, setTitle] = useState(book.title);
  const [author, setAuthor] = useState(book.author ?? "");
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [enriching, setEnriching] = useState(false);

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
          <div className="mx-auto aspect-[2/3] w-48 overflow-hidden rounded-lg bg-gray-100 dark:bg-gray-700">
            {book.cover_url ? (
              <img
                src={book.cover_url}
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
          </div>

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
          <ReviewEditor
            bookId={book.id}
            review={book.review}
            onSave={onReviewSave}
          />

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
