import { useState, useRef, useEffect, useCallback } from "react";

interface SortDropdownProps<T extends string> {
  value: T;
  onChange: (value: T) => void;
  options: { value: T; label: string }[];
}

export default function SortDropdown<T extends string>({
  value,
  onChange,
  options,
}: SortDropdownProps<T>) {
  const [open, setOpen] = useState(false);
  const [focusIndex, setFocusIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const currentLabel = options.find((o) => o.value === value)?.label ?? value;

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  // Scroll focused item into view
  useEffect(() => {
    if (open && focusIndex >= 0) {
      const items = listRef.current?.children;
      if (items?.[focusIndex]) {
        (items[focusIndex] as HTMLElement).scrollIntoView?.({ block: "nearest" });
      }
    }
  }, [open, focusIndex]);

  const toggle = useCallback(() => {
    setOpen((prev) => {
      if (!prev) {
        // Opening: focus the current value
        const idx = options.findIndex((o) => o.value === value);
        setFocusIndex(idx >= 0 ? idx : 0);
      }
      return !prev;
    });
  }, [options, value]);

  const select = useCallback(
    (v: T) => {
      onChange(v);
      setOpen(false);
    },
    [onChange],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!open) {
        if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
          e.preventDefault();
          toggle();
        }
        return;
      }

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setFocusIndex((i) => (i + 1) % options.length);
          break;
        case "ArrowUp":
          e.preventDefault();
          setFocusIndex((i) => (i - 1 + options.length) % options.length);
          break;
        case "Enter":
          e.preventDefault();
          if (focusIndex >= 0 && focusIndex < options.length) {
            select(options[focusIndex].value);
          }
          break;
        case "Escape":
          e.preventDefault();
          setOpen(false);
          break;
      }
    },
    [open, focusIndex, options, toggle, select],
  );

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        aria-label="Sort by"
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={toggle}
        onKeyDown={handleKeyDown}
        className="inline-flex items-center gap-1.5 rounded-md border border-gray-300 bg-white
          px-2.5 py-1.5 text-xs text-gray-700 transition-colors
          hover:bg-gray-50 focus:ring-2 focus:ring-amber-500 focus:outline-none
          dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
      >
        {currentLabel}
        <svg
          className={`h-3 w-3 transition-transform ${open ? "rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {open && (
        <ul
          ref={listRef}
          role="listbox"
          aria-label="Sort options"
          className="absolute right-0 z-20 mt-1 min-w-[10rem] overflow-auto rounded-md border
            border-gray-300 bg-white py-1 shadow-lg
            dark:border-gray-600 dark:bg-gray-800"
        >
          {options.map((opt, i) => {
            const isSelected = opt.value === value;
            const isFocused = i === focusIndex;
            return (
              <li
                key={opt.value}
                role="option"
                aria-selected={isSelected}
                onMouseEnter={() => setFocusIndex(i)}
                onMouseDown={(e) => {
                  e.preventDefault(); // prevent blur before click
                  select(opt.value);
                }}
                className={`cursor-pointer px-3 py-1.5 text-xs ${
                  isFocused
                    ? "bg-amber-50 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300"
                    : "text-gray-700 dark:text-gray-300"
                } ${isSelected ? "font-medium" : ""}`}
              >
                {opt.label}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
