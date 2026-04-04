import type { Book } from "../lib/api";
import { coverSrc } from "../lib/cover";
import RatingStars from "./RatingStars";
import { getStatusInfo } from "./StatusSelect";

interface LibraryListProps {
  books: Book[];
  onSelectBook: (book: Book) => void;
}

export default function LibraryList({
  books,
  onSelectBook,
}: LibraryListProps) {
  if (books.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center py-24 text-center">
        <div className="mb-4 text-6xl opacity-30">📚</div>
        <h2 className="text-lg font-medium text-gray-600 dark:text-gray-400">
          Your library is empty
        </h2>
        <p className="mt-1 text-sm text-gray-400 dark:text-gray-500">
          Add your first book with the + button above.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-1 p-6">
      {books.map((book) => {
        const statusInfo = getStatusInfo(book.status);
        return (
          <button
            key={book.id}
            type="button"
            onClick={() => onSelectBook(book)}
            className="flex items-center gap-4 rounded-lg px-3 py-2 text-left
              transition hover:bg-gray-100 dark:hover:bg-gray-800/60 cursor-pointer"
          >
            {/* Thumbnail */}
            <div className="h-14 w-10 flex-shrink-0 overflow-hidden rounded bg-gray-100 dark:bg-gray-700">
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
                  <span className="text-sm font-bold text-amber-700/60 dark:text-amber-400/60">
                    {book.title.charAt(0).toUpperCase()}
                  </span>
                </div>
              )}
            </div>

            {/* Title & author */}
            <div className="min-w-0 flex-1">
              <div className="truncate text-sm font-semibold text-gray-900 dark:text-gray-100">
                {book.title}
              </div>
              {book.author && (
                <div className="truncate text-sm text-gray-500 dark:text-gray-400">
                  {book.author}
                </div>
              )}
            </div>

            {/* Status badge */}
            {statusInfo.value && (
              <span
                className={`flex-shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium ${statusInfo.color}`}
              >
                {statusInfo.label}
              </span>
            )}

            {/* Rating */}
            {book.rating && (
              <div className="flex-shrink-0">
                <RatingStars rating={book.rating} onRate={() => {}} size="sm" />
              </div>
            )}

            {/* Date added */}
            <span className="flex-shrink-0 text-xs text-gray-400 dark:text-gray-500">
              {new Date(book.created_at).toLocaleDateString()}
            </span>
          </button>
        );
      })}
    </div>
  );
}
