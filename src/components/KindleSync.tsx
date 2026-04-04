import { useState, useRef, useEffect } from "react";
import {
  detectKindle,
  listKindleBooks,
  importKindleBooks,
  checkClippingsExist,
  importClippings,
  enrichBook,
  getBook,
} from "../lib/api";
import type { KindleBook } from "../lib/api";

type Phase =
  | "disconnected"
  | "detecting"
  | "connected"
  | "scanning"
  | "results"
  | "importing"
  | "done"
  | "clippings"
  | "importing_clippings"
  | "clippings_done";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const EXT_COLORS: Record<string, string> = {
  mobi: "bg-blue-600",
  azw: "bg-purple-600",
  azw3: "bg-purple-500",
  pdf: "bg-red-600",
  kfx: "bg-green-600",
};

export default function KindleSync({
  onClose,
  onImportComplete,
}: {
  onClose: () => void;
  onImportComplete: () => void;
}) {
  const [phase, setPhase] = useState<Phase>("disconnected");
  const [mountPath, setMountPath] = useState<string | null>(null);
  const [kindleBooks, setKindleBooks] = useState<KindleBook[]>([]);
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [importedCount, setImportedCount] = useState(0);
  const [clippingsCount, setClippingsCount] = useState(0);
  const [importedClippingsCount, setImportedClippingsCount] = useState(0);
  const [enrichProgress, setEnrichProgress] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const cancelledRef = useRef(false);

  useEffect(() => {
    return () => { cancelledRef.current = true; };
  }, []);

  const handleDetect = async () => {
    setPhase("detecting");
    setError(null);
    try {
      const path = await detectKindle();
      if (path) {
        setMountPath(path);
        setPhase("connected");
      } else {
        setPhase("disconnected");
        setError("No Kindle device found. Make sure it's connected via USB.");
      }
    } catch (e) {
      setPhase("disconnected");
      setError(String(e));
    }
  };

  const handleScan = async () => {
    if (!mountPath) return;
    setPhase("scanning");
    setError(null);
    try {
      const books = await listKindleBooks(mountPath);
      setKindleBooks(books);
      setSelected(new Set(books.map((_, i) => i)));
      setPhase("results");
    } catch (e) {
      setPhase("connected");
      setError(String(e));
    }
  };

  const handleImport = async () => {
    const toImport = kindleBooks.filter((_, i) => selected.has(i));
    if (toImport.length === 0) return;
    setPhase("importing");
    setError(null);
    try {
      const ids = await importKindleBooks(toImport);
      setImportedCount(ids.length);
      onImportComplete();

      // Background enrichment for books missing cover_url
      if (ids.length > 0) {
        (async () => {
          const booksToEnrich: number[] = [];
          for (const id of ids) {
            if (cancelledRef.current) return;
            try {
              const book = await getBook(id);
              if (!book.cover_url) booksToEnrich.push(id);
            } catch { /* skip */ }
          }
          for (let i = 0; i < booksToEnrich.length; i++) {
            if (cancelledRef.current) break;
            setEnrichProgress(`Enriching ${i + 1}/${booksToEnrich.length}...`);
            try {
              await enrichBook(booksToEnrich[i]);
            } catch { /* best effort */ }
          }
          if (!cancelledRef.current) {
            setEnrichProgress(null);
            onImportComplete();
          }
        })();
      }

      // Check for clippings
      if (mountPath) {
        try {
          const info = await checkClippingsExist(mountPath);
          if (info.exists && info.count > 0) {
            setClippingsCount(info.count);
            setPhase("clippings");
            return;
          }
        } catch {
          // Ignore clippings check failure, proceed to done
        }
      }
      setPhase("done");
    } catch (e) {
      setPhase("results");
      setError(String(e));
    }
  };

  const toggleAll = () => {
    if (selected.size === kindleBooks.length) {
      setSelected(new Set());
    } else {
      setSelected(new Set(kindleBooks.map((_, i) => i)));
    }
  };

  const toggleOne = (idx: number) => {
    const next = new Set(selected);
    if (next.has(idx)) next.delete(idx);
    else next.add(idx);
    setSelected(next);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div
        className="flex max-h-[80vh] w-full max-w-lg flex-col rounded-xl
          border border-gray-700 bg-gray-900 shadow-2xl"
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-gray-700 px-5 py-4">
          <h2 className="text-lg font-semibold text-gray-100">Kindle Sync</h2>
          <button
            type="button"
            onClick={onClose}
            className="text-gray-400 hover:text-gray-200"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-5 py-4">
          {error && (
            <div className="mb-4 rounded-lg bg-red-900/40 px-4 py-2 text-sm text-red-300">
              {error}
            </div>
          )}

          {phase === "disconnected" && (
            <div className="flex flex-col items-center gap-4 py-8 text-center">
              <svg className="h-16 w-16 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                  d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25" />
              </svg>
              <p className="text-gray-400">
                Connect your Kindle via USB to import books
              </p>
              <button
                type="button"
                onClick={handleDetect}
                className="rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                  text-white transition hover:bg-amber-700 active:scale-95"
              >
                Check Connection
              </button>
            </div>
          )}

          {phase === "detecting" && (
            <div className="flex flex-col items-center gap-3 py-12">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" />
              <p className="text-sm text-gray-400">Scanning for Kindle...</p>
            </div>
          )}

          {phase === "connected" && (
            <div className="flex flex-col items-center gap-4 py-8 text-center">
              <div className="rounded-full bg-green-900/40 p-3">
                <svg className="h-8 w-8 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <p className="text-gray-300">
                Kindle detected at <span className="font-mono text-amber-400">{mountPath}</span>
              </p>
              <button
                type="button"
                onClick={handleScan}
                className="rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                  text-white transition hover:bg-amber-700 active:scale-95"
              >
                Scan Books
              </button>
            </div>
          )}

          {phase === "scanning" && (
            <div className="flex flex-col items-center gap-3 py-12">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" />
              <p className="text-sm text-gray-400">Scanning Kindle library...</p>
            </div>
          )}

          {phase === "results" && (
            <div className="flex flex-col gap-3">
              {kindleBooks.length === 0 ? (
                <p className="py-8 text-center text-gray-400">
                  No supported books found on Kindle.
                </p>
              ) : (
                <>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-400">
                      {kindleBooks.length} book{kindleBooks.length !== 1 ? "s" : ""} found
                    </span>
                    <button
                      type="button"
                      onClick={toggleAll}
                      className="text-xs text-amber-400 hover:text-amber-300"
                    >
                      {selected.size === kindleBooks.length ? "Deselect all" : "Select all"}
                    </button>
                  </div>

                  <div className="flex flex-col gap-1">
                    {kindleBooks.map((book, idx) => (
                      <label
                        key={book.path}
                        className="flex cursor-pointer items-center gap-3 rounded-lg
                          px-3 py-2 transition hover:bg-gray-800"
                      >
                        <input
                          type="checkbox"
                          checked={selected.has(idx)}
                          onChange={() => toggleOne(idx)}
                          className="h-4 w-4 shrink-0 rounded border-gray-600 bg-gray-800
                            text-amber-500 accent-amber-500"
                        />
                        {book.cover_data ? (
                          <img
                            src={`data:image/jpeg;base64,${book.cover_data}`}
                            alt=""
                            className="h-12 w-9 shrink-0 rounded object-cover"
                          />
                        ) : (
                          <div className="flex h-12 w-9 shrink-0 items-center justify-center rounded bg-gray-700 text-[10px] text-gray-500">
                            No cover
                          </div>
                        )}
                        <div className="min-w-0 flex-1">
                          <p className="truncate text-sm font-medium text-gray-200">
                            {book.title}
                          </p>
                          {book.author && (
                            <p className="truncate text-xs text-gray-500">
                              {book.author}
                            </p>
                          )}
                          {book.publisher && (
                            <p className="truncate text-xs text-gray-600">
                              {book.publisher}
                            </p>
                          )}
                        </div>
                        <div className="flex shrink-0 flex-col items-end gap-1">
                          <div className="flex gap-1">
                            {book.cde_type && (
                              <span className="rounded bg-gray-700 px-1.5 py-0.5 text-[10px] font-medium uppercase text-gray-300">
                                {book.cde_type}
                              </span>
                            )}
                            <span
                              className={`rounded px-1.5 py-0.5 text-[10px] font-bold uppercase text-white ${
                                EXT_COLORS[book.extension] ?? "bg-gray-600"
                              }`}
                            >
                              {book.extension}
                            </span>
                          </div>
                          <span className="text-xs text-gray-500">
                            {formatBytes(book.size_bytes)}
                          </span>
                        </div>
                      </label>
                    ))}
                  </div>
                </>
              )}
            </div>
          )}

          {phase === "importing" && (
            <div className="flex flex-col items-center gap-3 py-12">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" />
              <p className="text-sm text-gray-400">Importing books...</p>
            </div>
          )}

          {phase === "clippings" && (
            <div className="flex flex-col items-center gap-4 py-8 text-center">
              <div className="rounded-full bg-amber-900/40 p-3">
                <svg className="h-8 w-8 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                    d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                </svg>
              </div>
              <p className="text-gray-300">
                Imported <span className="font-bold text-amber-400">{importedCount}</span> book
                {importedCount !== 1 ? "s" : ""}
              </p>
              <p className="text-gray-400 text-sm">
                Found <span className="font-bold text-amber-400">{clippingsCount}</span> highlight
                {clippingsCount !== 1 ? "s" : ""} in My Clippings.txt
              </p>
              {enrichProgress && (
                <p className="text-sm text-amber-400 animate-pulse">
                  {enrichProgress}
                </p>
              )}
              <button
                type="button"
                onClick={async () => {
                  if (!mountPath) return;
                  setPhase("importing_clippings");
                  setError(null);
                  try {
                    const count = await importClippings(mountPath);
                    setImportedClippingsCount(count);
                    setPhase("clippings_done");
                  } catch (e) {
                    setError(String(e));
                    setPhase("clippings_done");
                  }
                }}
                className="rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                  text-white transition hover:bg-amber-700 active:scale-95"
              >
                Import Highlights
              </button>
              <button
                type="button"
                onClick={() => setPhase("done")}
                className="text-sm text-gray-500 hover:text-gray-300"
              >
                Skip
              </button>
            </div>
          )}

          {phase === "importing_clippings" && (
            <div className="flex flex-col items-center gap-3 py-12">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-amber-500 border-t-transparent" />
              <p className="text-sm text-gray-400">Importing highlights...</p>
            </div>
          )}

          {(phase === "done" || phase === "clippings_done") && (
            <div className="flex flex-col items-center gap-4 py-8 text-center">
              <div className="rounded-full bg-green-900/40 p-3">
                <svg className="h-8 w-8 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <p className="text-gray-300">
                Imported <span className="font-bold text-amber-400">{importedCount}</span> book
                {importedCount !== 1 ? "s" : ""}
                {importedCount < selected.size && (
                  <span className="block text-xs text-gray-500 mt-1">
                    ({selected.size - importedCount} already in library)
                  </span>
                )}
              </p>
              {enrichProgress && (
                <p className="text-sm text-amber-400 animate-pulse">
                  {enrichProgress}
                </p>
              )}
              {phase === "clippings_done" && importedClippingsCount > 0 && (
                <p className="text-gray-300">
                  Imported <span className="font-bold text-amber-400">{importedClippingsCount}</span> highlight
                  {importedClippingsCount !== 1 ? "s" : ""}
                </p>
              )}
              <button
                type="button"
                onClick={onClose}
                className="rounded-lg bg-amber-600 px-5 py-2 text-sm font-medium
                  text-white transition hover:bg-amber-700 active:scale-95"
              >
                Close
              </button>
            </div>
          )}
        </div>

        {/* Footer action for results phase */}
        {phase === "results" && kindleBooks.length > 0 && selected.size > 0 && (
          <div className="border-t border-gray-700 px-5 py-3">
            <button
              type="button"
              onClick={handleImport}
              className="w-full rounded-lg bg-amber-600 py-2 text-sm font-medium
                text-white transition hover:bg-amber-700 active:scale-95"
            >
              Import {selected.size} Book{selected.size !== 1 ? "s" : ""}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
