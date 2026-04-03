# Blurb

A lightweight, local-first desktop app for managing your personal book library. Think Calibre, but modern and focused.

## Features

- **Library management** -- add books manually or by ISBN lookup, browse in a responsive cover grid
- **Ratings & reviews** -- rate books 1-5 stars and write personal reviews
- **Reading status** -- track books as Want to Read, Reading, Finished, or Abandoned
- **ISBN metadata lookup** -- auto-populate title, author, cover, and details from Open Library and Google Books
- **Kindle integration** -- detect a USB-connected Kindle, scan for books, and import them into your library
- **Local-first** -- all data stored in a local SQLite database. No cloud, no accounts, no tracking. You own your data.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri v2](https://tauri.app/) |
| Backend | Rust |
| Frontend | React 19 + TypeScript |
| Styling | Tailwind CSS v4 |
| Database | SQLite (via rusqlite) |
| Build tooling | Vite |

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Tauri CLI: `cargo install tauri-cli --version "^2"`
- Platform dependencies for Tauri: see [Tauri prerequisites](https://tauri.app/start/prerequisites/)

## Getting Started

```sh
# Clone the repo
git clone git@github.com:jfmyers9/blurb.git
cd blurb

# Install frontend dependencies
npm install

# Run in development mode
cargo tauri dev
```

The app will open in a native window. Hot-reload is enabled for both the frontend (Vite) and backend (Cargo).

## Building for Production

```sh
cargo tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
src/                          # React frontend
  components/
    AddBookForm.tsx           # Modal for adding new books
    BookCard.tsx              # Book tile in the library grid
    BookDetail.tsx            # Detail panel with editing, rating, review
    KindleSync.tsx            # Kindle device detection and import UI
    LibraryGrid.tsx           # Responsive cover grid layout
    RatingStars.tsx           # Clickable 1-5 star rating widget
    ReviewEditor.tsx          # Auto-saving review textarea
    StatusSelect.tsx          # Reading status dropdown
  lib/
    api.ts                    # Typed wrappers around Tauri IPC commands
  App.tsx                     # Main layout and state management

src-tauri/                    # Rust backend
  src/
    commands.rs               # Tauri IPC command handlers
    db.rs                     # SQLite schema, migrations, initialization
    kindle.rs                 # Kindle USB detection and file scanning
    lib.rs                    # App setup, state management
    metadata.rs               # ISBN lookup via Open Library / Google Books
    models.rs                 # Data structures (Book, BookInput)
```

## Data Storage

Blurb stores all data in a single SQLite file in your OS app data directory:

| OS | Location |
|----|----------|
| macOS | `~/Library/Application Support/com.blurb.app/` |
| Linux | `~/.local/share/com.blurb.app/` |
| Windows | `%APPDATA%\com.blurb.app\` |

The database is a standard SQLite file -- you can inspect it directly with any SQLite client.

## Kindle Integration

1. Connect your Kindle via USB
2. Click the device icon in the top bar
3. Click **Check Connection** to detect the Kindle
4. Click **Scan Books** to list books on the device
5. Select the books you want and click **Import Selected**

Imported books are added to your library with reading status set to "reading". The scanner parses book titles from filenames and strips Amazon metadata (ASINs, tags).

## License

MIT
