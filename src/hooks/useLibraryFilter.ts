import { useState, useMemo } from "react";
import type { Book, Shelf } from "../lib/api";
import type { FilterStatus, SortOption, ViewMode } from "../components/StatusFilterBar";

export function useLibraryFilter(
  books: Book[],
  shelves: Shelf[],
  shelfBookIdsMap: Record<number, number[]>
) {
  const [searchQuery, setSearchQuery] = useState("");
  const [activeStatus, setActiveStatus] = useState<FilterStatus>("all");
  const [sortBy, setSortBy] = useState<SortOption>("date_added");
  const [activeShelf, setActiveShelf] = useState<number | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>(
    () => (localStorage.getItem("blurb-view-mode") as ViewMode) || "grid"
  );
  const [minRating, setMinRating] = useState<number | null>(null);

  const searchableText = useMemo(() => {
    const map = new Map<number, string>();
    for (const book of books) {
      const shelfNames: string[] = [];
      for (const shelf of shelves) {
        if (shelfBookIdsMap[shelf.id]?.includes(book.id)) {
          shelfNames.push(shelf.name);
        }
      }
      map.set(
        book.id,
        [book.title, book.author ?? "", book.isbn ?? "", ...shelfNames]
          .join(" ")
          .toLowerCase()
      );
    }
    return map;
  }, [books, shelves, shelfBookIdsMap]);

  const filteredBooks = useMemo(() => {
    let filtered =
      activeStatus === "all"
        ? books
        : books.filter((b) => b.status === activeStatus);

    if (activeShelf !== null) {
      const bookIds = new Set(shelfBookIdsMap[activeShelf] ?? []);
      filtered = filtered.filter((b) => bookIds.has(b.id));
    }

    if (minRating !== null) {
      filtered = filtered.filter((b) => (b.rating ?? 0) >= minRating);
    }

    if (searchQuery) {
      const lowerQuery = searchQuery.toLowerCase();
      filtered = filtered.filter((b) =>
        searchableText.get(b.id)?.includes(lowerQuery)
      );
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
  }, [books, activeStatus, activeShelf, shelfBookIdsMap, sortBy, searchQuery, searchableText, minRating]);

  return {
    searchQuery,
    setSearchQuery,
    activeStatus,
    setActiveStatus,
    sortBy,
    setSortBy,
    activeShelf,
    setActiveShelf,
    viewMode,
    setViewMode,
    minRating,
    setMinRating,
    filteredBooks,
  };
}
