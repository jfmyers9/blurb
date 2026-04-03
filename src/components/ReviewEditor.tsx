import { useState, useEffect, useRef, useCallback } from "react";

interface ReviewEditorProps {
  bookId: number;
  review: string | null;
  onSave: (bookId: number, body: string) => Promise<void>;
}

export default function ReviewEditor({
  bookId,
  review,
  onSave,
}: ReviewEditorProps) {
  const [text, setText] = useState(review ?? "");
  const [saved, setSaved] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Reset text when book changes
  useEffect(() => {
    setText(review ?? "");
  }, [bookId, review]);

  const debouncedSave = useCallback(
    (value: string) => {
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(async () => {
        await onSave(bookId, value);
        setSaved(true);
        setTimeout(() => setSaved(false), 1500);
      }, 1000);
    },
    [bookId, onSave]
  );

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const handleChange = (value: string) => {
    setText(value);
    setSaved(false);
    debouncedSave(value);
  };

  return (
    <div>
      <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
        Review
      </label>
      <textarea
        value={text}
        onChange={(e) => handleChange(e.target.value)}
        rows={4}
        placeholder="Write your thoughts..."
        className="w-full rounded-md border border-gray-300 bg-white px-3 py-2
          text-sm text-gray-900 placeholder-gray-400
          dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100
          dark:placeholder-gray-500 focus:ring-2 focus:ring-amber-500
          focus:outline-none resize-y"
      />
      <div className="mt-1 h-4 text-xs">
        {saved && (
          <span className="text-green-600 dark:text-green-400">Saved</span>
        )}
      </div>
    </div>
  );
}
