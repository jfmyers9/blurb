fn stars(rating: i32) -> String {
    let filled = rating.clamp(0, 5) as usize;
    let empty = 5 - filled;
    "\u{2605}".repeat(filled) + &"\u{2606}".repeat(empty)
}

pub fn format_book_share(
    title: &str,
    author: Option<&str>,
    rating: Option<i32>,
    status: Option<&str>,
) -> String {
    let label = match status {
        Some("reading") => "Currently reading",
        Some("finished") => "Finished",
        Some("want_to_read") => "Want to read",
        Some("did_not_finish") => "Did not finish",
        _ => "Reading",
    };
    let mut out = format!("\u{1f4da} {label}: {title}");
    if let Some(a) = author {
        out.push_str(&format!(" by {a}"));
    }
    if let Some(r) = rating {
        out.push_str(&format!(" {}", stars(r)));
    }
    out
}

pub fn format_diary_share(
    title: &str,
    author: Option<&str>,
    rating: Option<i32>,
    date: &str,
    body: Option<&str>,
) -> String {
    let mut out = format!("\u{1f4d6} Finished: {title}");
    if let Some(a) = author {
        out.push_str(&format!(" by {a}"));
    }
    let mut meta_parts: Vec<String> = Vec::new();
    if let Some(r) = rating {
        meta_parts.push(stars(r));
    }
    meta_parts.push(date.to_string());
    out.push_str(&format!("\n{}", meta_parts.join(" | ")));
    if let Some(b) = body {
        if !b.is_empty() {
            out.push_str(&format!("\n\n\"{b}\""));
        }
    }
    out
}

pub fn format_shelf_share(shelf_name: &str, books: &[(String, Option<String>)]) -> String {
    let mut out = format!("\u{1f4da} My Shelf: {shelf_name}");
    for (i, (title, author)) in books.iter().enumerate() {
        out.push_str(&format!("\n{}. {title}", i + 1));
        if let Some(a) = author {
            out.push_str(&format!(" \u{2014} {a}"));
        }
    }
    out
}

pub fn format_stats_share(
    total: usize,
    finished: usize,
    pages: i64,
    avg_rating: Option<f64>,
) -> String {
    let mut out = format!("\u{1f4ca} My Reading Stats\n{total} books | {finished} finished");
    if pages > 0 {
        out.push_str(&format!(" | {pages} pages"));
    }
    if let Some(avg) = avg_rating {
        let rounded = avg.round() as i32;
        out.push_str(&format!(" | Avg {}", stars(rounded)));
    }
    out
}

#[cfg(test)]
#[path = "share_tests.rs"]
mod tests;
