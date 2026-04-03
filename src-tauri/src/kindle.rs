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

fn scan_dir(dir: &Path, books: &mut Vec<KindleBook>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
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

        let (title, author) = if let Some(idx) = stem.find(" - ") {
            (
                stem[..idx].to_string(),
                Some(stem[idx + 3..].to_string()),
            )
        } else {
            (stem, None)
        };

        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);

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
