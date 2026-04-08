use std::borrow::Cow;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};

#[cfg(test)]
#[path = "share_card_tests.rs"]
mod tests;

const CARD_WIDTH: u32 = 400;
const CARD_HEIGHT: u32 = 600;

// Star path from ui/rating_stars.rs (viewBox 0 0 20 20)
const STAR_PATH: &str = "M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z";

#[derive(Clone)]
pub enum ShareCardData {
    Book {
        title: String,
        author: String,
        rating: Option<i32>,
        cover_image_source: Option<String>,
    },
    Highlight {
        quote: String,
        book_title: String,
        author: String,
        rating: Option<i32>,
    },
}

pub fn copy_card_to_clipboard(data: &ShareCardData) -> Result<()> {
    let png_bytes = generate_card(data)?;
    let img = image::load_from_memory(&png_bytes)?.to_rgba8();
    let (width, height) = img.dimensions();
    arboard::Clipboard::new()?.set_image(arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: Cow::Owned(img.into_raw()),
    })?;
    Ok(())
}

pub fn generate_card(data: &ShareCardData) -> Result<Vec<u8>> {
    let svg = match data {
        ShareCardData::Book {
            title,
            author,
            rating,
            cover_image_source,
        } => build_book_svg(title, author, *rating, cover_image_source.as_deref()),
        ShareCardData::Highlight {
            quote,
            book_title,
            author,
            rating,
        } => build_highlight_svg(quote, book_title, author, *rating),
    };

    render_svg_to_png(&svg)
}

fn render_svg_to_png(svg: &str) -> Result<Vec<u8>> {
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(svg, &opt)?;
    let size = tree.size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width() as u32, size.height() as u32).unwrap();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    Ok(pixmap.encode_png()?)
}

fn build_book_svg(
    title: &str,
    author: &str,
    rating: Option<i32>,
    cover_source: Option<&str>,
) -> String {
    let cover_element = match cover_source.and_then(load_cover_image) {
        Some(data_uri) => format!(
            r#"<image x="120" y="30" width="160" height="240" href="{}" preserveAspectRatio="xMidYMid meet"/>"#,
            data_uri
        ),
        None => cover_fallback(title),
    };

    let title_lines = word_wrap(title, 25);
    let title_line_height = 26.0;
    let title_base_y = 310.0;
    let title_element = if title_lines.len() == 1 {
        format!(
            r#"<text x="200" y="{title_base_y}" text-anchor="middle" font-family="sans-serif" font-size="22" font-weight="bold" fill="white">{}</text>"#,
            escape_xml(&title_lines[0])
        )
    } else {
        let tspans: String = title_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let y = title_base_y + (i as f64 * title_line_height);
                format!(r#"<tspan x="200" y="{y}">{}</tspan>"#, escape_xml(line))
            })
            .collect::<Vec<_>>()
            .join("\n    ");
        format!(
            r#"<text text-anchor="middle" font-family="sans-serif" font-size="22" font-weight="bold" fill="white">
    {tspans}
  </text>"#
        )
    };

    let extra_lines = (title_lines.len() as f64 - 1.0) * title_line_height;
    let author_y = 340.0 + extra_lines;
    let author_escaped = escape_xml(author);

    let rating_svg = rating
        .map(|r| render_stars(r, 200.0, 460.0 + extra_lines))
        .unwrap_or_default();

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{CARD_WIDTH}" height="{CARD_HEIGHT}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#0f1b3d"/>
      <stop offset="100%" stop-color="#1a1a2e"/>
    </linearGradient>
  </defs>
  <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" fill="url(#bg)"/>
  {cover_element}
  {title_element}
  <text x="200" y="{author_y}" text-anchor="middle" font-family="sans-serif" font-size="16" fill="#a0aec0">{author_escaped}</text>
  {rating_svg}
  <text x="200" y="575" text-anchor="middle" font-family="sans-serif" font-size="12" fill="#4a5568" opacity="0.6">Blurb</text>
</svg>"##
    )
}

fn clamp_lines(mut lines: Vec<String>, max: usize) -> Vec<String> {
    if lines.len() > max {
        lines.truncate(max);
        if let Some(last) = lines.last_mut() {
            last.push_str("...");
        }
    }
    lines
}

fn build_highlight_svg(quote: &str, book_title: &str, author: &str, rating: Option<i32>) -> String {
    let lines = clamp_lines(word_wrap(quote, 35), 12);
    let start_y = 120.0;
    let line_height = 28.0;
    let tspans: String = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let y = start_y + (i as f64 * line_height);
            format!(r#"<tspan x="200" y="{y}">{}</tspan>"#, escape_xml(line))
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    let quote_bottom = start_y + (lines.len() as f64 * line_height) + 20.0;
    let title_y = quote_bottom + 30.0;
    let author_y = title_y + 28.0;
    let rating_y = author_y + 30.0;

    let title_escaped = escape_xml(book_title);
    let author_escaped = escape_xml(author);

    let rating_svg = rating
        .map(|r| render_stars(r, 200.0, rating_y))
        .unwrap_or_default();

    // Open/close quote marks positioned around the quote block
    let open_quote_y = start_y - 30.0;
    let close_quote_y = quote_bottom - 10.0;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{CARD_WIDTH}" height="{CARD_HEIGHT}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#0f1b3d"/>
      <stop offset="100%" stop-color="#1a1a2e"/>
    </linearGradient>
  </defs>
  <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" fill="url(#bg)"/>
  <text x="40" y="{open_quote_y}" font-family="serif" font-size="60" fill="#4a5568" opacity="0.4">&#x201C;</text>
  <text text-anchor="middle" font-family="serif" font-size="18" fill="#e2e8f0" font-style="italic">
    {tspans}
  </text>
  <text x="360" y="{close_quote_y}" font-family="serif" font-size="60" fill="#4a5568" opacity="0.4">&#x201D;</text>
  <text x="200" y="{title_y}" text-anchor="middle" font-family="sans-serif" font-size="16" font-weight="bold" fill="white">{title_escaped}</text>
  <text x="200" y="{author_y}" text-anchor="middle" font-family="sans-serif" font-size="14" fill="#a0aec0">{author_escaped}</text>
  {rating_svg}
  <text x="200" y="575" text-anchor="middle" font-family="sans-serif" font-size="12" fill="#4a5568" opacity="0.6">Blurb</text>
</svg>"##
    )
}

fn cover_fallback(title: &str) -> String {
    let letter = title
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    format!(
        r##"<rect x="120" y="30" width="160" height="240" rx="8" fill="#2d3748"/>
  <text x="200" y="175" text-anchor="middle" font-family="sans-serif" font-size="72" font-weight="bold" fill="white">{letter}</text>"##
    )
}

fn render_stars(rating: i32, center_x: f64, y: f64) -> String {
    let star_size = 16.0;
    let gap = 2.0;
    let total_width = 5.0 * star_size + 4.0 * gap;
    let start_x = center_x - total_width / 2.0;

    let stars: String = (0..5)
        .map(|i| {
            let x = start_x + (i as f64) * (star_size + gap);
            let fill = if i < rating { "#f6ad55" } else { "#4a5568" };
            format!(
                r#"<g transform="translate({x},{y}) scale({scale})"><path d="{STAR_PATH}" fill="{fill}"/></g>"#,
                scale = star_size / 20.0,
            )
        })
        .collect::<Vec<_>>()
        .join("\n  ");

    stars
}

fn load_cover_image(source: &str) -> Option<String> {
    let bytes = if source.starts_with("http") {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?
            .get(source)
            .send()
            .ok()?
            .bytes()
            .ok()?
            .to_vec()
    } else {
        std::fs::read(source).ok()?
    };

    let mime = detect_image_mime(&bytes)?;
    let b64 = STANDARD.encode(&bytes);
    Some(format!("data:{mime};base64,{b64}"))
}

fn detect_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF") {
        Some("image/gif")
    } else if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

pub fn word_wrap(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() > max_chars {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line.push(' ');
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
