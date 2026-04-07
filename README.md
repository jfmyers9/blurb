# Blurb

A lightweight, local-first desktop app for managing your personal book library. Think Calibre, but modern and focused.

## Features

- **Library management** -- add books manually or by ISBN lookup, browse in a responsive cover grid or list view
- **Ratings & reviews** -- rate books 1-5 stars and write personal reviews
- **Reading diary** -- keep a reading journal with per-book diary entries
- **Reading status** -- track books as Want to Read, Reading, Finished, or Abandoned
- **Shelves** -- organize books into custom collections
- **ISBN metadata lookup** -- auto-populate title, author, cover, and details from Open Library and Google Books
- **Kindle integration** -- detect a USB-connected Kindle, scan for books, and import them with highlights and clippings
- **Command palette** -- keyboard-driven navigation and search
- **Local-first** -- all data stored in a local SQLite database. No cloud, no accounts, no tracking. You own your data.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Dioxus 0.7](https://dioxuslabs.com/) (desktop) |
| Language | Rust |
| Styling | Tailwind CSS |
| Database | SQLite (via rusqlite, bundled) |
| Build tooling | Dioxus CLI |

Fully Rust-native -- no JavaScript, no Node.js, no web server.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Dioxus CLI: `cargo install dioxus-cli`

## Getting Started

```sh
# Clone the repo
git clone git@github.com:jfmyers9/blurb.git
cd blurb

# Run in development mode (hot reload)
dx serve
```

The app will open in a native desktop window.

## Building for Production

```sh
dx build --release
```

## Logging

Blurb writes structured logs to a daily rolling file:

| OS | Log Location |
|----|-------------|
| macOS | `~/Library/Logs/com.blurb.app/blurb.log.YYYY-MM-DD` |

**Viewing logs:**

```sh
# Tail the current log file
tail -f ~/Library/Logs/com.blurb.app/blurb.log.$(date +%Y-%m-%d)

# Or open in Console.app
open -a Console ~/Library/Logs/com.blurb.app/
```

**Changing log level:**

Set the `RUST_LOG` environment variable before launching:

```sh
RUST_LOG=debug dx serve    # verbose output for development
RUST_LOG=trace dx serve    # maximum detail
```

Default levels: `info` in release builds, `debug` in development.

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

Imported books are added to your library with reading status set to "want to read". The scanner parses MOBI files for metadata and can import highlights from the Kindle clippings file.

## License

MIT
