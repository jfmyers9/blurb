use base64::Engine;
use mobi::headers::ExthRecord;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindleBook {
    pub filename: String,
    pub path: String,
    pub title: String,
    pub author: Option<String>,
    pub asin: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub cover_data: Option<String>,
    pub cde_type: Option<String>,
    pub extension: String,
    pub size_bytes: u64,
}

const KINDLE_EXTENSIONS: &[&str] = &["mobi", "azw", "azw3", "pdf", "kfx"];
const MOBI_EXTENSIONS: &[&str] = &["mobi", "azw", "azw3"];

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

pub(crate) fn find_documents_dir(mount: &Path) -> Option<std::path::PathBuf> {
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
    scan_dir(&docs_dir, &mut books, 0);
    books
}

const SKIP_DIRS: &[&str] = &[".sdr", ".tmp"];
const MIN_BOOK_SIZE: u64 = 10_000; // 10KB — skip metadata stubs

fn exth_string(exth: &mobi::headers::ExtHeader, record: ExthRecord) -> Option<String> {
    exth.get_record(record)?
        .first()
        .map(|d| String::from_utf8_lossy(d).to_string())
        .filter(|s| !s.is_empty())
}

fn is_image_content(content: &[u8]) -> bool {
    if content.len() < 4 {
        return false;
    }
    let bytes = &content[..4];
    bytes != b"FLIS"
        && bytes != b"FCIS"
        && bytes != b"SRCS"
        && bytes != b"RESC"
        && bytes != b"BOUN"
        && bytes != b"FDST"
        && bytes != b"DATP"
        && bytes != b"AUDI"
        && bytes != b"VIDE"
        && bytes != b"\xe9\x8e\r\n"
}

fn read_mobi_metadata(path: &Path) -> Option<MobiFields> {
    let m = mobi::Mobi::from_path(path).ok()?;

    let exth = &m.metadata.exth;

    let title = {
        let t = exth_string(exth, ExthRecord::Title).unwrap_or_else(|| m.title());
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    };

    let author = exth_string(exth, ExthRecord::Author).or_else(|| m.author());
    let asin = exth_string(exth, ExthRecord::Asin);
    let isbn = exth_string(exth, ExthRecord::Isbn);
    let publisher = exth_string(exth, ExthRecord::Publisher).or_else(|| m.publisher());
    let description = exth_string(exth, ExthRecord::Description).or_else(|| m.description());
    let published_date = exth_string(exth, ExthRecord::PublishDate).or_else(|| m.publish_date());
    let language = exth_string(exth, ExthRecord::Language).or_else(|| {
        let lang = format!("{:?}", m.language());
        if lang == "Unknown" {
            None
        } else {
            Some(lang)
        }
    });
    let cde_type = exth_string(exth, ExthRecord::Cdetype);

    // Extract cover image via CoverOffset EXTH record
    let cover_data = extract_cover_base64(&m);

    Some(MobiFields {
        title,
        author,
        asin,
        isbn,
        publisher,
        description,
        published_date,
        language,
        cover_data,
        cde_type,
    })
}

fn extract_cover_base64(m: &mobi::Mobi) -> Option<String> {
    let exth = &m.metadata.exth;
    let cover_offset_data = exth.get_record(ExthRecord::CoverOffset)?;
    let offset_bytes = cover_offset_data.first()?;
    if offset_bytes.len() < 4 {
        return None;
    }
    let cover_offset = u32::from_be_bytes([
        offset_bytes[0],
        offset_bytes[1],
        offset_bytes[2],
        offset_bytes[3],
    ]) as usize;

    let first_image = m.metadata.mobi.first_image_index as usize;
    let record_index = first_image + cover_offset;

    let records = m.raw_records();
    let all: Vec<_> = records.into_iter().collect();
    let record = all.get(record_index)?;

    if !is_image_content(record.content) {
        return None;
    }

    let encoded = base64::engine::general_purpose::STANDARD.encode(record.content);
    Some(encoded)
}

struct MobiFields {
    title: Option<String>,
    author: Option<String>,
    asin: Option<String>,
    isbn: Option<String>,
    publisher: Option<String>,
    description: Option<String>,
    published_date: Option<String>,
    language: Option<String>,
    cover_data: Option<String>,
    cde_type: Option<String>,
}

fn scan_dir(dir: &Path, books: &mut Vec<KindleBook>, depth: u32) {
    if depth > 10 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.is_symlink() {
                continue;
            }
            let dir_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            if SKIP_DIRS.iter().any(|s| dir_name.ends_with(s)) {
                continue;
            }
            scan_dir(&path, books, depth + 1);
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

        if stem.starts_with("CR!") || stem.starts_with('.') {
            continue;
        }

        // Try EXTH metadata for MOBI/AZW/AZW3
        let mobi_meta = if MOBI_EXTENSIONS.contains(&ext.as_str()) {
            read_mobi_metadata(&path)
        } else {
            None
        };

        let (
            title,
            author,
            asin,
            isbn,
            publisher,
            description,
            published_date,
            language,
            cover_data,
            cde_type,
        ) = if let Some(meta) = mobi_meta {
            let (fallback_title, fallback_author, fallback_asin) = parse_kindle_filename(&stem);
            (
                meta.title.unwrap_or(fallback_title),
                meta.author.or(fallback_author),
                meta.asin.or(fallback_asin),
                meta.isbn,
                meta.publisher,
                meta.description,
                meta.published_date,
                meta.language,
                meta.cover_data,
                meta.cde_type,
            )
        } else {
            let (title, author, asin) = parse_kindle_filename(&stem);
            (
                title, author, asin, None, None, None, None, None, None, None,
            )
        };

        books.push(KindleBook {
            filename,
            path: path.to_string_lossy().to_string(),
            title,
            author,
            asin,
            isbn,
            publisher,
            description,
            published_date,
            language,
            cover_data,
            cde_type,
            extension: ext,
            size_bytes,
        });
    }
}

fn strip_asin_suffix(name: &str) -> (&str, Option<&str>) {
    // Amazon appends ASINs like "_B0XXXXXXXXX" or " (B0XXXXXXXXX)"
    // ASINs are 10 chars starting with B0
    if let Some(idx) = name.rfind("_B0") {
        let suffix = &name[idx + 1..];
        if suffix.len() >= 10 {
            if let Some(asin_slice) = suffix.get(..10) {
                if asin_slice.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return (name[..idx].trim(), Some(asin_slice));
                }
            }
        }
    }
    if let Some(idx) = name.rfind(" (B0") {
        if name.ends_with(')') {
            let asin_start = idx + 2;
            if let Some(asin_slice) = name.get(asin_start..asin_start + 10) {
                if asin_slice.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return (name[..idx].trim(), Some(asin_slice));
                }
            }
        }
    }
    // Also strip trailing _EBOK, _PDOC, etc.
    for tag in &["_EBOK", "_PDOC", "_EBSP"] {
        if let Some(stripped) = name.strip_suffix(tag) {
            return (stripped.trim(), None);
        }
    }
    (name, None)
}

fn parse_kindle_filename(stem: &str) -> (String, Option<String>, Option<String>) {
    let (stem, asin) = strip_asin_suffix(stem);

    // Pattern: "Title - Author"
    if let Some(idx) = stem.find(" - ") {
        let (title, asin2) = strip_asin_suffix(stem[..idx].trim());
        let (author, asin3) = strip_asin_suffix(stem[idx + 3..].trim());
        let asin = asin.or(asin2).or(asin3).map(|s| s.to_string());
        let title = title.to_string();
        if !author.is_empty() {
            return (title, Some(author.to_string()), asin);
        }
        return (title, None, asin);
    }

    // Replace underscores with spaces for readability
    let cleaned = stem.replace('_', " ");
    (
        cleaned.trim().to_string(),
        None,
        asin.map(|s| s.to_string()),
    )
}

#[cfg(test)]
#[path = "kindle_tests.rs"]
mod tests;
