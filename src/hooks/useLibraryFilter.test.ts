import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useLibraryFilter } from "./useLibraryFilter";
import type { Book, Shelf } from "../lib/api";

function makeBook(overrides: Partial<Book> = {}): Book {
  return {
    id: 1,
    title: "Test Book",
    author: "Author",
    isbn: null,
    asin: null,
    cover_url: null,
    description: null,
    publisher: null,
    published_date: null,
    page_count: null,
    created_at: "2024-01-01",
    updated_at: "2024-01-01",
    rating: null,
    status: null,
    review: null,
    ...overrides,
  };
}

const books: Book[] = [
  makeBook({ id: 1, title: "Alpha", author: "Zara", isbn: "111", rating: 5, status: "reading", created_at: "2024-03-01" }),
  makeBook({ id: 2, title: "Beta", author: "Yolanda", isbn: "222", rating: 3, status: "finished", created_at: "2024-02-01" }),
  makeBook({ id: 3, title: "Gamma", author: "Xander", isbn: "333", rating: 2, status: "want_to_read", created_at: "2024-01-01" }),
  makeBook({ id: 4, title: "Delta", author: "Wendy", isbn: null, rating: null, status: "reading", created_at: "2024-04-01" }),
];

const shelves: Shelf[] = [
  { id: 10, name: "Fiction", created_at: "2024-01-01" },
  { id: 20, name: "Science", created_at: "2024-01-01" },
];

const shelfBookIdsMap: Record<number, number[]> = {
  10: [1, 2],
  20: [3],
};

beforeEach(() => {
  const store: Record<string, string> = {};
  vi.stubGlobal("localStorage", {
    getItem(key: string) { return store[key] ?? null; },
    setItem(key: string, value: string) { store[key] = value; },
    removeItem(key: string) { delete store[key]; },
  });
});

describe("useLibraryFilter", () => {
  describe("search", () => {
    it("matches title", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setSearchQuery("alpha"));
      expect(result.current.filteredBooks.map((b) => b.id)).toEqual([1]);
    });

    it("matches author", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setSearchQuery("xander"));
      expect(result.current.filteredBooks.map((b) => b.id)).toEqual([3]);
    });

    it("matches ISBN", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setSearchQuery("222"));
      expect(result.current.filteredBooks.map((b) => b.id)).toEqual([2]);
    });

    it("matches shelf name", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setSearchQuery("science"));
      expect(result.current.filteredBooks.map((b) => b.id)).toEqual([3]);
    });
  });

  describe("minRating", () => {
    it("includes books at threshold and excludes below", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setMinRating(3));
      const ids = result.current.filteredBooks.map((b) => b.id);
      expect(ids).toContain(1); // rating 5
      expect(ids).toContain(2); // rating 3
      expect(ids).not.toContain(3); // rating 2
      expect(ids).not.toContain(4); // rating null → 0
    });
  });

  describe("composed filters", () => {
    it("status + search + rating + shelf all active simultaneously", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => {
        result.current.setActiveStatus("reading");
        result.current.setSearchQuery("fiction"); // shelf name → books 1,2
        result.current.setMinRating(3);
        result.current.setActiveShelf(10);
      });
      // book 1: reading, on shelf 10 ("Fiction"), rating 5 → passes all
      // book 4: reading but not on shelf 10
      expect(result.current.filteredBooks.map((b) => b.id)).toEqual([1]);
    });
  });

  describe("sorting", () => {
    it("sort by title produces alphabetical order", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setSortBy("title"));
      expect(result.current.filteredBooks.map((b) => b.title)).toEqual([
        "Alpha", "Beta", "Delta", "Gamma",
      ]);
    });

    it("sort by author produces alphabetical order", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setSortBy("author"));
      expect(result.current.filteredBooks.map((b) => b.author)).toEqual([
        "Wendy", "Xander", "Yolanda", "Zara",
      ]);
    });

    it("sort by rating produces descending order", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      act(() => result.current.setSortBy("rating"));
      expect(result.current.filteredBooks.map((b) => b.rating)).toEqual([
        5, 3, 2, null,
      ]);
    });

    it("sort by date_added produces descending chronological order", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      // date_added is the default sort
      expect(result.current.filteredBooks.map((b) => b.id)).toEqual([4, 1, 2, 3]);
    });
  });

  describe("viewMode", () => {
    it("defaults to grid when localStorage is empty", () => {
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      expect(result.current.viewMode).toBe("grid");
    });

    it("reads from localStorage", () => {
      localStorage.setItem("blurb-view-mode", "list");
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      expect(result.current.viewMode).toBe("list");
    });

    it("defaults to grid for invalid localStorage value", () => {
      localStorage.setItem("blurb-view-mode", "table");
      const { result } = renderHook(() => useLibraryFilter(books, shelves, shelfBookIdsMap));
      expect(result.current.viewMode).toBe("grid");
    });
  });
});
