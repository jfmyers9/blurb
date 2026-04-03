import { invoke } from "@tauri-apps/api/core";

export interface Book {
  id: number;
  title: string;
  author: string | null;
  isbn: string | null;
  asin: string | null;
  cover_url: string | null;
  description: string | null;
  publisher: string | null;
  published_date: string | null;
  page_count: number | null;
  created_at: string;
  updated_at: string;
  rating: number | null;
  status: string | null;
  review: string | null;
}

export type ReadingStatus =
  | "want_to_read"
  | "reading"
  | "finished"
  | "abandoned";

export async function listBooks(): Promise<Book[]> {
  return invoke<Book[]>("list_books");
}

export async function getBook(id: number): Promise<Book> {
  return invoke<Book>("get_book", { id });
}

export async function addBook(params: {
  title: string;
  author?: string | null;
  isbn?: string | null;
  asin?: string | null;
  cover_url?: string | null;
  description?: string | null;
  publisher?: string | null;
  published_date?: string | null;
  page_count?: number | null;
}): Promise<number> {
  return invoke<number>("add_book", {
    title: params.title,
    author: params.author ?? null,
    isbn: params.isbn ?? null,
    asin: params.asin ?? null,
    cover_url: params.cover_url ?? null,
    description: params.description ?? null,
    publisher: params.publisher ?? null,
    published_date: params.published_date ?? null,
    page_count: params.page_count ?? null,
  });
}

export async function updateBook(params: {
  id: number;
  title: string;
  author?: string | null;
  isbn?: string | null;
  asin?: string | null;
  cover_url?: string | null;
  description?: string | null;
  publisher?: string | null;
  published_date?: string | null;
  page_count?: number | null;
}): Promise<Book> {
  return invoke<Book>("update_book", {
    id: params.id,
    title: params.title,
    author: params.author ?? null,
    isbn: params.isbn ?? null,
    asin: params.asin ?? null,
    cover_url: params.cover_url ?? null,
    description: params.description ?? null,
    publisher: params.publisher ?? null,
    published_date: params.published_date ?? null,
    page_count: params.page_count ?? null,
  });
}

export async function deleteBook(id: number): Promise<void> {
  return invoke<void>("delete_book", { id });
}

export async function setRating(
  book_id: number,
  score: number
): Promise<void> {
  return invoke<void>("set_rating", { book_id, score });
}

export async function setReadingStatus(
  book_id: number,
  status: string
): Promise<void> {
  return invoke<void>("set_reading_status", { book_id, status });
}

export interface BookMetadata {
  title: string | null;
  author: string | null;
  cover_url: string | null;
  description: string | null;
  publisher: string | null;
  published_date: string | null;
  page_count: number | null;
}

export async function lookupIsbn(isbn: string): Promise<BookMetadata> {
  return invoke<BookMetadata>("lookup_isbn", { isbn });
}

export async function searchCovers(query: string): Promise<BookMetadata[]> {
  return invoke<BookMetadata[]>("search_covers", { query });
}

export async function saveReview(
  book_id: number,
  body: string
): Promise<void> {
  return invoke<void>("save_review", { book_id, body });
}

export interface KindleBook {
  filename: string;
  path: string;
  title: string;
  author: string | null;
  extension: string;
  size_bytes: number;
}

export async function detectKindle(): Promise<string | null> {
  return invoke<string | null>("detect_kindle");
}

export async function listKindleBooks(
  mount_path: string
): Promise<KindleBook[]> {
  return invoke<KindleBook[]>("list_kindle_books", { mount_path });
}

export async function importKindleBooks(
  books: KindleBook[]
): Promise<number[]> {
  return invoke<number[]>("import_kindle_books", { books });
}

export async function uploadCover(
  book_id: number,
  source_path: string
): Promise<string> {
  return invoke<string>("upload_cover", { book_id, source_path });
}
