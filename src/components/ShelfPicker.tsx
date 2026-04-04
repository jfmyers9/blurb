import { useState, useRef, useEffect } from "react";
import type { Shelf } from "../lib/api";

interface ShelfPickerProps {
  shelves: Shelf[];
  bookShelfIds: number[];
  onAdd: (shelfId: number) => void;
  onRemove: (shelfId: number) => void;
  onCreate: (name: string) => Promise<Shelf>;
}

export default function ShelfPicker({
  shelves,
  bookShelfIds,
  onAdd,
  onRemove,
  onCreate,
}: ShelfPickerProps) {
  const [input, setInput] = useState("");
  const [open, setOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  const bookShelves = shelves.filter((s) => bookShelfIds.includes(s.id));
  const trimmed = input.trim().toLowerCase();
  const suggestions = shelves.filter(
    (s) =>
      !bookShelfIds.includes(s.id) &&
      s.name.toLowerCase().includes(trimmed)
  );
  const exactMatch = shelves.some(
    (s) => s.name.toLowerCase() === trimmed
  );
  const showCreate = trimmed.length > 0 && !exactMatch;

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const handleCreate = async () => {
    if (!trimmed || creating) return;
    setCreating(true);
    try {
      const shelf = await onCreate(input.trim());
      onAdd(shelf.id);
      setInput("");
      setOpen(false);
    } catch {
      // Creation failed — leave input as-is so user can retry
    } finally {
      setCreating(false);
    }
  };

  const handleSelect = (shelfId: number) => {
    onAdd(shelfId);
    setInput("");
    setOpen(false);
  };

  return (
    <div ref={wrapperRef}>
      {/* Chips */}
      {bookShelves.length > 0 && (
        <div className="mb-2 flex flex-wrap gap-1.5">
          {bookShelves.map((s) => (
            <span
              key={s.id}
              className="inline-flex items-center gap-1 rounded-full bg-amber-100 px-2.5 py-0.5
                text-xs font-medium text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
            >
              {s.name}
              <button
                type="button"
                onClick={() => onRemove(s.id)}
                className="ml-0.5 rounded-full p-0.5 hover:bg-amber-200 dark:hover:bg-amber-800/40"
              >
                <svg className="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </span>
          ))}
        </div>
      )}

      {/* Combo input */}
      <div className="relative">
        <input
          type="text"
          value={input}
          onChange={(e) => {
            setInput(e.target.value);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && showCreate && suggestions.length === 0) {
              e.preventDefault();
              handleCreate();
            } else if (e.key === "Enter" && suggestions.length === 1) {
              e.preventDefault();
              handleSelect(suggestions[0].id);
            }
          }}
          placeholder="Add to shelf..."
          className="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm
            text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
            focus:ring-2 focus:ring-amber-500 focus:outline-none"
        />

        {open && (suggestions.length > 0 || showCreate) && (
          <div
            className="absolute z-10 mt-1 w-full rounded-md border border-gray-200 bg-white
              shadow-lg dark:border-gray-700 dark:bg-gray-800"
          >
            {suggestions.map((s) => (
              <button
                key={s.id}
                type="button"
                onClick={() => handleSelect(s.id)}
                className="block w-full px-3 py-1.5 text-left text-sm text-gray-900
                  hover:bg-amber-50 dark:text-gray-100 dark:hover:bg-amber-900/20"
              >
                {s.name}
              </button>
            ))}
            {showCreate && (
              <button
                type="button"
                onClick={handleCreate}
                disabled={creating}
                className="block w-full border-t border-gray-100 px-3 py-1.5 text-left text-sm
                  font-medium text-amber-600 hover:bg-amber-50
                  dark:border-gray-700 dark:text-amber-400 dark:hover:bg-amber-900/20
                  disabled:opacity-50"
              >
                {creating ? "Creating..." : `Create "${input.trim()}"`}
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
