import { useRef, useCallback } from "react";
import type { KeyboardEvent } from "react";
import type { Book } from "../lib/api";
import BookCard from "./BookCard";

const CARD_MIN_WIDTH = 180;

interface LibraryGridProps {
  books: Book[];
  onSelectBook: (book: Book) => void;
}

export default function LibraryGrid({
  books,
  onSelectBook,
}: LibraryGridProps) {
  const gridRef = useRef<HTMLDivElement>(null);
  const cardRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const setCardRef = useCallback(
    (index: number) => (el: HTMLButtonElement | null) => {
      cardRefs.current[index] = el;
    },
    [],
  );

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const { key } = e;
    if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(key))
      return;

    const currentIndex = cardRefs.current.findIndex(
      (ref) => ref === document.activeElement,
    );
    if (currentIndex === -1) return;

    const total = books.length;
    const cols = gridRef.current
      ? Math.floor(gridRef.current.getBoundingClientRect().width / CARD_MIN_WIDTH)
      : 1;

    let next = currentIndex;
    if (key === "ArrowLeft") {
      next = Math.max(0, currentIndex - 1);
    } else if (key === "ArrowRight") {
      next = Math.min(total - 1, currentIndex + 1);
    } else if (key === "ArrowUp") {
      next = currentIndex - cols;
    } else if (key === "ArrowDown") {
      next = currentIndex + cols;
    }

    if (next >= 0 && next < total && next !== currentIndex) {
      e.preventDefault();
      cardRefs.current[next]?.focus();
    }
  };

  cardRefs.current.length = books.length;

  return (
    <div
      ref={gridRef}
      onKeyDown={handleKeyDown}
      className="grid gap-4 p-6"
      style={{
        gridTemplateColumns: `repeat(auto-fill, minmax(${CARD_MIN_WIDTH}px, 1fr))`,
      }}
    >
      {books.map((book, i) => (
        <BookCard
          key={book.id}
          ref={setCardRef(i)}
          book={book}
          onClick={() => onSelectBook(book)}
        />
      ))}
    </div>
  );
}
