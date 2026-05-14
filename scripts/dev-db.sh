#!/usr/bin/env bash
set -euo pipefail

PROD_APP_ID="com.blurb.app"
DEV_APP_ID="com.blurb.app.dev"
DB_NAME="blurb.db"

usage() {
	cat <<'USAGE'
Usage: scripts/dev-db.sh <command> [--yes]

Commands:
  run          Launch Blurb against the isolated development database
  path         Print development data, database, covers, and log paths
  reset        Delete development DB/WAL/SHM, covers, and logs (confirmation required)
  clone-prod   Copy production DB and covers into development (confirmation required)
  seed         Create a small deterministic development dataset

Environment:
  BLURB_PROFILE=dev is used for run.
  BLURB_DATA_DIR is intentionally ignored by this script so reset/clone never target production.
USAGE
}

data_home() {
	case "$(uname -s)" in
	Darwin) printf '%s\n' "$HOME/Library/Application Support" ;;
	Linux) printf '%s\n' "${XDG_DATA_HOME:-$HOME/.local/share}" ;;
	*) printf '%s\n' "${APPDATA:-$HOME/.local/share}" ;;
	esac
}

log_root() {
	# Mirrors src/data/db.rs + src/logging.rs for named profiles.
	printf '%s\n' "$HOME/Library/Logs"
}

app_dir() { printf '%s/%s\n' "$(data_home)" "$1"; }
log_dir() { printf '%s/%s\n' "$(log_root)" "$1"; }
db_path() { printf '%s/%s\n' "$(app_dir "$1")" "$DB_NAME"; }
covers_dir() { printf '%s/covers\n' "$(app_dir "$1")"; }

sql_quote() {
	local value=${1//\'/\'\'}
	printf "'%s'" "$value"
}

require_sqlite3() {
	if ! command -v sqlite3 >/dev/null 2>&1; then
		echo "sqlite3 is required" >&2
		exit 1
	fi
}

confirm_dev_action() {
	local prompt=$1
	local yes=${2:-}
	local dev_dir prod_dir
	dev_dir=$(app_dir "$DEV_APP_ID")
	prod_dir=$(app_dir "$PROD_APP_ID")

	if [[ "$dev_dir" == "$prod_dir" || "$DEV_APP_ID" == "$PROD_APP_ID" ]]; then
		echo "Refusing: development path resolves to production path" >&2
		exit 1
	fi

	if [[ "$yes" == "--yes" ]]; then
		return 0
	fi

	printf '%s Type "dev" to continue: ' "$prompt" >&2
	local answer
	read -r answer
	if [[ "$answer" != "dev" ]]; then
		echo "Canceled" >&2
		exit 1
	fi
}

print_paths() {
	cat <<EOF
profile: dev
app_id: $DEV_APP_ID
app_dir: $(app_dir "$DEV_APP_ID")
db: $(db_path "$DEV_APP_ID")
covers: $(covers_dir "$DEV_APP_ID")
logs: $(log_dir "$DEV_APP_ID")
EOF
}

reset_dev() {
	local yes=${1:-}
	confirm_dev_action "Delete the development database, covers, and logs?" "$yes"

	local dev_db dev_covers dev_logs
	dev_db=$(db_path "$DEV_APP_ID")
	dev_covers=$(covers_dir "$DEV_APP_ID")
	dev_logs=$(log_dir "$DEV_APP_ID")

	rm -f "$dev_db" "$dev_db-wal" "$dev_db-shm"
	rm -rf "$dev_covers" "$dev_logs"
	mkdir -p "$(app_dir "$DEV_APP_ID")"
	echo "Reset development database state."
}

copy_covers() {
	local src=$1 dest=$2
	rm -rf "$dest"
	if [[ ! -d "$src" ]]; then
		mkdir -p "$dest"
		return 0
	fi
	mkdir -p "$(dirname "$dest")"
	if command -v rsync >/dev/null 2>&1; then
		mkdir -p "$dest"
		rsync -a --delete "$src/" "$dest/"
	else
		cp -R "$src" "$dest"
	fi
}

clone_prod() {
	local yes=${1:-}
	require_sqlite3
	confirm_dev_action "Replace the development database with a clone of production?" "$yes"

	local prod_db dev_db prod_covers dev_covers
	prod_db=$(db_path "$PROD_APP_ID")
	dev_db=$(db_path "$DEV_APP_ID")
	prod_covers=$(covers_dir "$PROD_APP_ID")
	dev_covers=$(covers_dir "$DEV_APP_ID")

	if [[ ! -f "$prod_db" ]]; then
		echo "Production DB not found: $prod_db" >&2
		exit 1
	fi

	mkdir -p "$(dirname "$dev_db")"
	rm -f "$dev_db" "$dev_db-wal" "$dev_db-shm"
	sqlite3 "$prod_db" ".backup $(sql_quote "$dev_db")"
	copy_covers "$prod_covers" "$dev_covers"

	local prod_sql dev_sql
	prod_sql=$(sql_quote "$prod_covers")
	dev_sql=$(sql_quote "$dev_covers")
	sqlite3 "$dev_db" "UPDATE books SET cover_url = replace(cover_url, $prod_sql, $dev_sql) WHERE cover_url LIKE $prod_sql || '%';"

	echo "Cloned production database into development."
}

seed_dev() {
	require_sqlite3
	mkdir -p "$(app_dir "$DEV_APP_ID")" "$(covers_dir "$DEV_APP_ID")"
	local dev_db
	dev_db=$(db_path "$DEV_APP_ID")

	sqlite3 "$dev_db" <<'SQL'
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS books(
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT,
    isbn TEXT,
    asin TEXT,
    cover_url TEXT,
    description TEXT,
    publisher TEXT,
    published_date TEXT,
    page_count INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS reading_status(
    id INTEGER PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK(status IN ('want_to_read','reading','finished','abandoned')),
    started_at TEXT,
    finished_at TEXT,
    updated_at TEXT NOT NULL,
    UNIQUE(book_id)
);
CREATE TABLE IF NOT EXISTS ratings(
    id INTEGER PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    score INTEGER NOT NULL CHECK(score BETWEEN 1 AND 5),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(book_id)
);
CREATE TABLE IF NOT EXISTS reviews(
    id INTEGER PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(book_id)
);
CREATE TABLE IF NOT EXISTS shelves(
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS book_shelves(
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    shelf_id INTEGER NOT NULL REFERENCES shelves(id) ON DELETE CASCADE,
    UNIQUE(book_id, shelf_id)
);
CREATE TABLE IF NOT EXISTS highlights(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    location_start INTEGER,
    location_end INTEGER,
    page INTEGER,
    clip_type TEXT NOT NULL CHECK(clip_type IN ('highlight','note','bookmark')),
    clipped_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(book_id, text, location_start)
);
CREATE TABLE IF NOT EXISTS diary_entries(
    id INTEGER PRIMARY KEY,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    body TEXT,
    rating INTEGER CHECK(rating BETWEEN 1 AND 5),
    entry_date TEXT NOT NULL DEFAULT (date('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_diary_entries_book_date ON diary_entries(book_id, entry_date DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_reading_status_book_id ON reading_status(book_id);
CREATE INDEX IF NOT EXISTS idx_ratings_book_id ON ratings(book_id);
CREATE INDEX IF NOT EXISTS idx_reviews_book_id ON reviews(book_id);
CREATE INDEX IF NOT EXISTS idx_highlights_book_id ON highlights(book_id);
CREATE INDEX IF NOT EXISTS idx_book_shelves_book_id ON book_shelves(book_id);
CREATE INDEX IF NOT EXISTS idx_book_shelves_shelf_id ON book_shelves(shelf_id);
PRAGMA user_version = 3;
DELETE FROM highlights;
DELETE FROM diary_entries;
DELETE FROM book_shelves;
DELETE FROM shelves;
DELETE FROM reviews;
DELETE FROM ratings;
DELETE FROM reading_status;
DELETE FROM books;
INSERT INTO books (id, title, author, isbn, description, publisher, published_date, page_count, created_at, updated_at) VALUES
  (1, 'Dev Import Fixture', 'A. Tester', '9780000000001', 'Seeded book for import and review testing.', 'Blurb Fixtures', '2026', 240, datetime('now'), datetime('now')),
  (2, 'Duplicate Candidate', 'B. Reviewer', '9780000000002', 'Use this title to test duplicate import behavior.', 'Blurb Fixtures', '2025', 180, datetime('now'), datetime('now')),
  (3, 'Highlights Sandbox', 'C. Kindle', NULL, 'Use this book for clippings and highlights.', 'Blurb Fixtures', '2024', 320, datetime('now'), datetime('now'));
INSERT INTO reading_status (book_id, status, started_at, finished_at, updated_at) VALUES
  (1, 'reading', date('now', '-3 day'), NULL, datetime('now')),
  (2, 'finished', date('now', '-20 day'), date('now', '-10 day'), datetime('now')),
  (3, 'want_to_read', NULL, NULL, datetime('now'));
INSERT INTO ratings (book_id, score, created_at, updated_at) VALUES (2, 4, datetime('now'), datetime('now'));
INSERT INTO reviews (book_id, body, created_at, updated_at) VALUES (2, 'Seed review text for card/review UI testing.', datetime('now'), datetime('now'));
INSERT INTO shelves (id, name) VALUES (1, 'dev-fixtures'), (2, 'import-tests');
INSERT INTO book_shelves (book_id, shelf_id) VALUES (1, 1), (2, 1), (2, 2);
INSERT INTO highlights (book_id, text, location_start, location_end, page, clip_type, clipped_at) VALUES
  (3, 'Seed highlight for browser testing.', 128, 132, NULL, 'highlight', datetime('now')),
  (3, 'Seed note for clippings import review.', NULL, NULL, 42, 'note', datetime('now'));
INSERT INTO diary_entries (book_id, body, rating, entry_date) VALUES
  (1, 'Seed diary entry for development database flow.', NULL, date('now'));
SQL

	echo "Seeded development database: $dev_db"
}

cmd=${1:-}
case "$cmd" in
run)
	shift
	BLURB_PROFILE=dev dx serve "$@"
	;;
path)
	print_paths
	;;
reset)
	shift || true
	reset_dev "${1:-}"
	;;
clone-prod)
	shift || true
	clone_prod "${1:-}"
	;;
seed)
	seed_dev
	;;
-h | --help | help | "")
	usage
	;;
*)
	echo "Unknown command: $cmd" >&2
	usage >&2
	exit 1
	;;
esac
