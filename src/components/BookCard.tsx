import { forwardRef } from "react";
import type { Book } from "../lib/api";
import { coverSrc } from "../lib/cover";
import RatingStars from "./RatingStars";
import { getStatusInfo } from "./StatusSelect";

interface BookCardProps {
  book: Book;
  onClick: () => void;
}

const BookCard = forwardRef<HTMLButtonElement, BookCardProps>(
  function BookCard({ book, onClick }, ref) {
    const statusInfo = getStatusInfo(book.status);

    return (
      <button
        ref={ref}
        type="button"
        onClick={onClick}
        className="group flex flex-col overflow-hidden rounded-lg bg-white
          shadow-sm ring-1 ring-gray-200 transition hover:shadow-md
          hover:ring-amber-300 dark:bg-gray-800 dark:ring-gray-700
          dark:hover:ring-amber-600 cursor-pointer text-left
          focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none"
      >
      {/* Cover */}
      <div className="relative aspect-[2/3] w-full overflow-hidden bg-gray-100 dark:bg-gray-700">
        {book.cover_url ? (
          <img
            src={coverSrc(book.cover_url)}
            alt={book.title}
            className="h-full w-full object-cover transition group-hover:scale-105"
          />
        ) : (
          <div
            className="flex h-full w-full items-center justify-center
              bg-gradient-to-br from-amber-100 to-orange-200
              dark:from-amber-900/40 dark:to-orange-900/40"
          >
            <span className="text-4xl font-bold text-amber-700/60 dark:text-amber-400/60">
              {book.title.charAt(0).toUpperCase()}
            </span>
          </div>
        )}
        {/* Status badge */}
        {statusInfo.value && (
          <span
            className={`absolute top-2 right-2 rounded-full px-2 py-0.5
              text-[10px] font-medium ${statusInfo.color}`}
          >
            {statusInfo.label}
          </span>
        )}
      </div>

      {/* Info */}
      <div className="flex flex-1 flex-col gap-1 p-3">
        <h3 className="line-clamp-2 text-sm font-semibold text-gray-900 dark:text-gray-100">
          {book.title}
        </h3>
        {book.author && (
          <p className="line-clamp-1 text-xs text-gray-500 dark:text-gray-400">
            {book.author}
          </p>
        )}
        {book.rating && (
          <div className="mt-auto pt-1">
            <RatingStars rating={book.rating} onRate={() => {}} size="sm" />
          </div>
        )}
      </div>
    </button>
  );
  }
);

export default BookCard;
