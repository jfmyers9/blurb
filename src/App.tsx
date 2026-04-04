import { useState, useEffect, useCallback, useMemo } from "react";
import "./App.css";
import {
  listBooks,
  addBook,
  updateBook,
  deleteBook,
  setRating,
  setReadingStatus,
  lookupIsbn,
  listShelves,
  listBookShelves,
  addBookToShelf,
  removeBookFromShelf,
  createShelf,
  renameShelf,
  deleteShelf,
  listAllShelfBookIds,
} from "./lib/api";
import type { Book, Shelf } from "./lib/api";
import LibraryGrid from "./components/LibraryGrid";
import BookDetail from "./components/BookDetail";
import AddBookForm from "./components/AddBookForm";
import KindleSync from "./components/KindleSync";
import ReviewPage from "./components/ReviewPage";
import StatusFilterBar from "./components/StatusFilterBar";
import type { FilterStatus, SortOption } from "./components/StatusFilterBar";

function App() {
  const [books, setBooks] = useState<Book[]>([]);
  const [selectedBook, setSelectedBook] = useState<Book | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [showKindle, setShowKindle] = useState(false);
  const [editingReviewBookId, setEditingReviewBookId] = useState<number | null>(null);
  const [activeStatus, setActiveStatus] = useState<FilterStatus>("all");
  const [sortBy, setSortBy] = useState<SortOption>("date_added");
  const [shelves, setShelves] = useState<Shelf[]>([]);
  const [bookShelfMap, setBookShelfMap] = useState<Record<number, number[]>>({});
  const [activeShelf, setActiveShelf] = useState<number | null>(null);
  const [shelfBookIdsMap, setShelfBookIdsMap] = useState<Record<number, number[]>>({});

  const filteredBooks = useMemo(() => {
    let filtered =
      activeStatus === "all"
        ? books
        : books.filter((b) => b.status === activeStatus);

    if (activeShelf !== null) {
      const bookIds = new Set(shelfBookIdsMap[activeShelf] ?? []);
      filtered = filtered.filter((b) => bookIds.has(b.id));
    }

    filtered = [...filtered].sort((a, b) => {
      switch (sortBy) {
        case "title":
          return a.title.localeCompare(b.title);
        case "author":
          return (a.author ?? "").localeCompare(b.author ?? "");
        case "rating":
          return (b.rating ?? 0) - (a.rating ?? 0);
        case "date_added":
        default:
          return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      }
    });

    return filtered;
  }, [books, activeStatus, activeShelf, shelfBookIdsMap, sortBy]);

  const refresh = useCallback(async () => {
    const data = await listBooks();
    setBooks(data);
    return data;
  }, []);

  const refreshShelves = useCallback(async () => {
    const [s, pairs] = await Promise.all([listShelves(), listAllShelfBookIds()]);
    setShelves(s);
    const map: Record<number, number[]> = {};
    for (const [shelfId, bookId] of pairs) {
      (map[shelfId] ??= []).push(bookId);
    }
    setShelfBookIdsMap(map);
    return s;
  }, []);

  const loadBookShelves = useCallback(async (bookId: number) => {
    const bs = await listBookShelves(bookId);
    setBookShelfMap((prev) => ({ ...prev, [bookId]: bs.map((s) => s.id) }));
  }, []);

  useEffect(() => {
    refresh();
    refreshShelves();
  }, [refresh, refreshShelves]);

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
    const data = await refresh();
    setSelectedBook(data.find((b) => b.id === id) ?? null);
  };

  const handleDelete = async (id: number) => {
    await deleteBook(id);
    setSelectedBook(null);
    await refresh();
    await refreshShelves();
  };

  const handleRate = async (bookId: number, score: number) => {
    await setRating(bookId, score);
    const data = await refresh();
    setSelectedBook(data.find((b) => b.id === bookId) ?? null);
  };

  const handleStatusChange = async (bookId: number, status: string) => {
    await setReadingStatus(bookId, status);
    const data = await refresh();
    setSelectedBook(data.find((b) => b.id === bookId) ?? null);
  };

  const handleAddToShelf = async (bookId: number, shelfId: number) => {
    await addBookToShelf(bookId, shelfId);
    await loadBookShelves(bookId);
    setShelfBookIdsMap((prev) => ({
      ...prev,
      [shelfId]: [...(prev[shelfId] ?? []), bookId],
    }));
  };

  const handleRemoveFromShelf = async (bookId: number, shelfId: number) => {
    await removeBookFromShelf(bookId, shelfId);
    await loadBookShelves(bookId);
    setShelfBookIdsMap((prev) => ({
      ...prev,
      [shelfId]: (prev[shelfId] ?? []).filter((id) => id !== bookId),
    }));
  };

  const handleCreateShelf = async (name: string) => {
    const id = await createShelf(name);
    const updated = await refreshShelves();
    const created = updated.find((s) => s.id === id);
    if (!created) throw new Error(`Shelf ${id} not found after creation`);
    return created;
  };

  const handleRenameShelf = async (shelfId: number, newName: string) => {
    await renameShelf(shelfId, newName);
    setShelves((prev) =>
      prev.map((s) => (s.id === shelfId ? { ...s, name: newName } : s))
    );
  };

  const handleDeleteShelf = async (shelfId: number, bookCount: number) => {
    const msg =
      bookCount > 0
        ? `This shelf contains ${bookCount} book${bookCount === 1 ? "" : "s"}. Books won't be deleted. Delete shelf?`
        : "Delete this shelf?";
    if (!window.confirm(msg)) return;
    await deleteShelf(shelfId);
    setShelves((prev) => prev.filter((s) => s.id !== shelfId));
    setShelfBookIdsMap((prev) => {
      const next = { ...prev };
      delete next[shelfId];
      return next;
    });
    setBookShelfMap((prev) => {
      const next: Record<number, number[]> = {};
      for (const [bookId, shelfIds] of Object.entries(prev)) {
        const filtered = shelfIds.filter((id) => id !== shelfId);
        if (filtered.length > 0) next[Number(bookId)] = filtered;
      }
      return next;
    });
    if (activeShelf === shelfId) setActiveShelf(null);
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
    const data = await refresh();
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
    const data = await refresh();
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
        <StatusFilterBar
          books={books}
          activeStatus={activeStatus}
          onStatusChange={setActiveStatus}
          sortBy={sortBy}
          onSortChange={setSortBy}
          shelves={shelves}
          activeShelf={activeShelf}
          onShelfChange={setActiveShelf}
          shelfBookCounts={Object.fromEntries(
            shelves.map((s) => [s.id, shelfBookIdsMap[s.id]?.length ?? 0])
          )}
          onRenameShelf={handleRenameShelf}
          onDeleteShelf={handleDeleteShelf}
        />
        <LibraryGrid
          books={filteredBooks}
          onSelectBook={(book) => setSelectedBook(book)}
        />
      </main>

      {/* Detail panel */}
      {selectedBook && (
        <BookDetail
          key={selectedBook.id}
          book={selectedBook}
          onClose={() => setSelectedBook(null)}
          onUpdate={handleUpdate}
          onDelete={handleDelete}
          onRate={handleRate}
          onStatusChange={handleStatusChange}
          onEditReview={(bookId) => setEditingReviewBookId(bookId)}
          onLookup={handleLookup}
          onCoverChange={handleCoverChange}
          shelves={shelves}
          bookShelfIds={bookShelfMap[selectedBook.id] ?? []}
          onAddToShelf={handleAddToShelf}
          onRemoveFromShelf={handleRemoveFromShelf}
          onCreateShelf={handleCreateShelf}
          onLoadBookShelves={loadBookShelves}
        />
      )}

      {/* Review overlay */}
      {editingReviewBookId !== null && (
        <ReviewPage
          bookId={editingReviewBookId}
          onClose={() => setEditingReviewBookId(null)}
          onSave={async () => {
            const data = await refresh();
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
