import { useState, useEffect, useCallback } from "react";
import "./App.css";
import {
  listBooks,
  addBook,
  updateBook,
  deleteBook,
  setRating,
  setReadingStatus,
  lookupIsbn,
} from "./lib/api";
import type { Book } from "./lib/api";
import LibraryGrid from "./components/LibraryGrid";
import BookDetail from "./components/BookDetail";
import AddBookForm from "./components/AddBookForm";
import KindleSync from "./components/KindleSync";
import ReviewPage from "./components/ReviewPage";

function App() {
  const [books, setBooks] = useState<Book[]>([]);
  const [selectedBook, setSelectedBook] = useState<Book | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [showKindle, setShowKindle] = useState(false);
  const [editingReviewBookId, setEditingReviewBookId] = useState<number | null>(null);

  const refresh = useCallback(async () => {
    const data = await listBooks();
    setBooks(data);
  }, []);

  // Refresh selected book from latest list
  const refreshAndSync = useCallback(async () => {
    const data = await listBooks();
    setBooks(data);
    return data;
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleAdd = async (data: {
    title: string;
    author: string;
    isbn: string;
    cover_url?: string | null;
    description?: string | null;
    publisher?: string | null;
    published_date?: string | null;
    page_count?: number | null;
  }) => {
    await addBook({
      title: data.title,
      author: data.author || null,
      isbn: data.isbn || null,
      cover_url: data.cover_url ?? null,
      description: data.description ?? null,
      publisher: data.publisher ?? null,
      published_date: data.published_date ?? null,
      page_count: data.page_count ?? null,
    });
    await refresh();
  };

  const handleUpdate = async (
    id: number,
    title: string,
    author: string | null
  ) => {
    const book = books.find((b) => b.id === id);
    if (!book) return;
    await updateBook({
      id,
      title,
      author,
      isbn: book.isbn,
      asin: book.asin,
      cover_url: book.cover_url,
      description: book.description,
      publisher: book.publisher,
      published_date: book.published_date,
      page_count: book.page_count,
    });
    const data = await refreshAndSync();
    setSelectedBook(data.find((b) => b.id === id) ?? null);
  };

  const handleDelete = async (id: number) => {
    await deleteBook(id);
    setSelectedBook(null);
    await refresh();
  };

  const handleRate = async (bookId: number, score: number) => {
    await setRating(bookId, score);
    const data = await refreshAndSync();
    setSelectedBook(data.find((b) => b.id === bookId) ?? null);
  };

  const handleStatusChange = async (bookId: number, status: string) => {
    await setReadingStatus(bookId, status);
    const data = await refreshAndSync();
    setSelectedBook(data.find((b) => b.id === bookId) ?? null);
  };

  const handleCoverChange = async (bookId: number, coverUrl: string) => {
    const book = books.find((b) => b.id === bookId);
    if (!book) return;
    await updateBook({
      id: bookId,
      title: book.title,
      author: book.author,
      isbn: book.isbn,
      asin: book.asin,
      cover_url: coverUrl,
      description: book.description,
      publisher: book.publisher,
      published_date: book.published_date,
      page_count: book.page_count,
    });
    const data = await refreshAndSync();
    setSelectedBook(data.find((b) => b.id === bookId) ?? null);
  };

  const handleLookup = async (bookId: number) => {
    const book = books.find((b) => b.id === bookId);
    if (!book?.isbn) return;
    const meta = await lookupIsbn(book.isbn);
    await updateBook({
      id: bookId,
      title: meta.title ?? book.title,
      author: meta.author ?? book.author,
      isbn: book.isbn,
      asin: book.asin,
      cover_url: meta.cover_url ?? book.cover_url,
      description: meta.description ?? book.description,
      publisher: meta.publisher ?? book.publisher,
      published_date: meta.published_date ?? book.published_date,
      page_count: meta.page_count ?? book.page_count,
    });
    const data = await refreshAndSync();
    setSelectedBook(data.find((b) => b.id === bookId) ?? null);
  };

  return (
    <div className="flex min-h-screen flex-col bg-gray-50 dark:bg-gray-950">
      {/* Top bar */}
      <header
        className="sticky top-0 z-30 flex items-center justify-between
          border-b border-gray-200 bg-white/80 px-6 py-3 backdrop-blur
          dark:border-gray-800 dark:bg-gray-900/80"
      >
        <h1 className="text-xl font-bold tracking-tight text-gray-900 dark:text-gray-100">
          Blurb
        </h1>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setShowKindle(true)}
            className="flex h-9 w-9 items-center justify-center rounded-full
              text-gray-400 transition hover:bg-gray-100 hover:text-gray-700
              dark:hover:bg-gray-800 dark:hover:text-gray-200"
            title="Kindle sync"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                d="M10.5 19.5h3m-6.75 2.25h10.5a2.25 2.25 0 002.25-2.25v-15a2.25 2.25 0 00-2.25-2.25H6.75A2.25 2.25 0 004.5 4.5v15a2.25 2.25 0 002.25 2.25z" />
            </svg>
          </button>
          <button
            type="button"
            onClick={() => setShowAddForm(true)}
            className="flex h-9 w-9 items-center justify-center rounded-full
              bg-amber-600 text-white shadow-sm transition hover:bg-amber-700
              active:scale-95"
            title="Add book"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
      </header>

      {/* Main */}
      <main className="flex-1">
        <LibraryGrid
          books={books}
          onSelectBook={(book) => setSelectedBook(book)}
        />
      </main>

      {/* Detail panel */}
      {selectedBook && (
        <BookDetail
          book={selectedBook}
          onClose={() => setSelectedBook(null)}
          onUpdate={handleUpdate}
          onDelete={handleDelete}
          onRate={handleRate}
          onStatusChange={handleStatusChange}
          onEditReview={(bookId) => setEditingReviewBookId(bookId)}
          onLookup={handleLookup}
          onCoverChange={handleCoverChange}
        />
      )}

      {/* Review overlay */}
      {editingReviewBookId !== null && (
        <ReviewPage
          bookId={editingReviewBookId}
          onClose={() => setEditingReviewBookId(null)}
          onSave={async () => {
            const data = await refreshAndSync();
            setSelectedBook(data.find((b) => b.id === editingReviewBookId) ?? null);
          }}
        />
      )}

      {/* Kindle sync */}
      {showKindle && (
        <KindleSync
          onClose={() => setShowKindle(false)}
          onImportComplete={refresh}
        />
      )}

      {/* Add book modal */}
      <AddBookForm
        open={showAddForm}
        onClose={() => setShowAddForm(false)}
        onAdd={handleAdd}
      />
    </div>
  );
}

export default App;
