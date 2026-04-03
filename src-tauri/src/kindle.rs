use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindleBook {
    pub filename: String,
    pub path: String,
    pub title: String,
    pub author: Option<String>,
    pub extension: String,
    pub size_bytes: u64,
}

const KINDLE_EXTENSIONS: &[&str] = &["mobi", "azw", "azw3", "pdf", "kfx"];

pub fn detect_kindle() -> Option<String> {
    let volumes = Path::new("/Volumes");
    if !volumes.exists() {
        return None;
    }

    let entries = fs::read_dir(volumes).ok()?;
    for entry in entries.flatten() {
        let mount = entry.path();
        if !mount.is_dir() {
            continue;
        }

        let has_documents = has_subdir_case_insensitive(&mount, "documents");
        let has_system = has_subdir_case_insensitive(&mount, "system");

        if has_documents && has_system {
            return mount.to_str().map(|s| s.to_string());
        }
    }

    None
}

fn has_subdir_case_insensitive(parent: &Path, name: &str) -> bool {
    let Ok(entries) = fs::read_dir(parent) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            && e.file_name().to_string_lossy().eq_ignore_ascii_case(name)
    })
}

fn find_documents_dir(mount: &Path) -> Option<std::path::PathBuf> {
    let entries = fs::read_dir(mount).ok()?;
    for entry in entries.flatten() {
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("documents")
        {
            return Some(entry.path());
        }
    }
    None
}

pub fn list_kindle_books(mount_path: &str) -> Vec<KindleBook> {
    let mount = Path::new(mount_path);
    let Some(docs_dir) = find_documents_dir(mount) else {
        return Vec::new();
    };

    let mut books = Vec::new();
    scan_dir(&docs_dir, &mut books);
    books
}

const SKIP_DIRS: &[&str] = &[".sdr", ".tmp"];
const MIN_BOOK_SIZE: u64 = 10_000; // 10KB — skip metadata stubs

fn scan_dir(dir: &Path, books: &mut Vec<KindleBook>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            if SKIP_DIRS.iter().any(|s| dir_name.ends_with(s)) {
                continue;
            }
            scan_dir(&path, books);
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !KINDLE_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }

        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if size_bytes < MIN_BOOK_SIZE {
            continue;
        }

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Skip files that look like Amazon internal IDs (e.g. "CR!XXXX", "GutenbergXXX")
        if stem.starts_with("CR!") || stem.starts_with('.') {
            continue;
        }

        let (title, author) = parse_kindle_filename(&stem);

        books.push(KindleBook {
            filename,
            path: path.to_string_lossy().to_string(),
            title,
            author,
            extension: ext,
            size_bytes,
        });
    }
}

fn strip_asin_suffix(name: &str) -> &str {
    // Amazon appends ASINs like "_B0XXXXXXXXX" or " (B0XXXXXXXXX)"
    // ASINs are 10 chars starting with B0
    if let Some(idx) = name.rfind("_B0") {
        let suffix = &name[idx + 1..];
        if suffix.len() >= 10 && suffix[..10].chars().all(|c| c.is_ascii_alphanumeric()) {
            return name[..idx].trim();
        }
    }
    if let Some(idx) = name.rfind(" (B0") {
        if name.ends_with(')') {
            return name[..idx].trim();
        }
    }
    // Also strip trailing _EBOK, _PDOC, etc.
    for tag in &["_EBOK", "_PDOC", "_EBSP"] {
        if let Some(stripped) = name.strip_suffix(tag) {
            return stripped.trim();
        }
    }
    name
}

fn parse_kindle_filename(stem: &str) -> (String, Option<String>) {
    let stem = strip_asin_suffix(stem);

    // Pattern: "Title - Author"
    if let Some(idx) = stem.find(" - ") {
        let title = strip_asin_suffix(stem[..idx].trim()).to_string();
        let author = strip_asin_suffix(stem[idx + 3..].trim()).to_string();
        if !author.is_empty() {
            return (title, Some(author));
        }
        return (title, None);
    }

    // Replace underscores with spaces for readability
    let cleaned = stem.replace('_', " ");
    (cleaned.trim().to_string(), None)
}
