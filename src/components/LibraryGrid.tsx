import type { Book } from "../lib/api";
import BookCard from "./BookCard";

interface LibraryGridProps {
  books: Book[];
  onSelectBook: (book: Book) => void;
}

export default function LibraryGrid({
  books,
  onSelectBook,
}: LibraryGridProps) {
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
    <div
      className="grid gap-4 p-6"
      style={{
        gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
      }}
    >
      {books.map((book) => (
        <BookCard key={book.id} book={book} onClick={() => onSelectBook(book)} />
      ))}
    </div>
  );
}
