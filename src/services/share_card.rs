use std::borrow::Cow;
use std::path::PathBuf;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};

#[cfg(test)]
#[path = "share_card_tests.rs"]
mod tests;

const CARD_WIDTH: u32 = 400;
const CARD_HEIGHT: u32 = 600;
const SCALE_FACTOR: u32 = 3;

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

pub fn save_card_to_file(data: &ShareCardData, name: &str) -> Result<PathBuf> {
    let png_bytes = generate_card(data)?;
    let path = std::env::temp_dir().join(format!("blurb-{name}.png"));
    std::fs::write(&path, &png_bytes)?;
    Ok(path)
}

pub fn open_share_sheet(data: &ShareCardData, name: &str) -> Result<()> {
    let path = save_card_to_file(data, name)?;
    show_share_sheet(path);
    Ok(())
}

#[cfg(target_os = "macos")]
fn show_share_sheet(path: PathBuf) {
    use objc2::rc::Retained;
    use objc2::{AnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSSharingServicePicker};
    use objc2_foundation::{NSArray, NSString, NSURL};

    let path_str = path.to_string_lossy().to_string();

    dispatch::Queue::main().exec_async(move || {
        // Safety: we're on the main thread via dispatch::Queue::main()
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        let ns_path = NSString::from_str(&path_str);
        let url = NSURL::fileURLWithPath(&ns_path);
        let items: Retained<NSArray> = NSArray::from_retained_slice(&[url.into()]);
        let picker = unsafe {
            NSSharingServicePicker::initWithItems(NSSharingServicePicker::alloc(), &items)
        };

        let app = NSApplication::sharedApplication(mtm);
        if let Some(window) = app.keyWindow() {
            if let Some(view) = window.contentView() {
                let frame = view.frame();
                // Anchor near center-bottom of the window
                let rect = objc2_foundation::NSRect::new(
                    objc2_foundation::NSPoint::new(frame.size.width / 2.0, 0.0),
                    objc2_foundation::NSSize::new(1.0, 1.0),
                );
                picker.showRelativeToRect_ofView_preferredEdge(
                    rect,
                    &view,
                    objc2_foundation::NSRectEdge::MinY,
                );
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn show_share_sheet(path: PathBuf) {
    let _ = opener::open(&path);
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

fn bundled_fonts_dir() -> Option<PathBuf> {
    // Tauri macOS bundle: <exe>/../Resources/assets/fonts/
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| {
        p.parent()?
            .parent()
            .map(|p| p.join("Resources/assets/fonts"))
    }) {
        if dir.is_dir() {
            return Some(dir);
        }
    }
    // Dev: CARGO_MANIFEST_DIR/assets/fonts/
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    if dir.is_dir() {
        return Some(dir);
    }
    None
}

fn load_app_icon() -> Option<String> {
    // Tauri macOS bundle: <exe>/../Resources/assets/icons/32x32.png
    let paths = [
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()?
                    .parent()
                    .map(|p| p.join("Resources/assets/icons/32x32.png"))
            })
            .unwrap_or_default(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/icons/32x32.png"),
    ];
    for path in &paths {
        if let Ok(bytes) = std::fs::read(path) {
            let b64 = STANDARD.encode(&bytes);
            return Some(format!("data:image/png;base64,{b64}"));
        }
    }
    None
}

fn branded_footer(icon_data_uri: Option<&str>, dark_theme: bool, y_start: f64) -> String {
    let (separator_color, text_fill, url_fill) = if dark_theme {
        ("rgba(255,255,255,0.1)", "white", "#94a3b8")
    } else {
        ("rgba(0,0,0,0.08)", "#1a1a2e", "#78716c")
    };
    let sep_y = y_start;
    let icon_y = sep_y + 12.0;
    let text_y = icon_y + 16.0;
    let url_y = text_y + 18.0;
    let url_line = format!(
        r#"<text x="200" y="{url_y}" text-anchor="middle" font-family="Inter, sans-serif" font-size="13" fill="{url_fill}">github.com/jfmyers9/blurb</text>"#
    );
    match icon_data_uri {
        Some(uri) => {
            format!(
                r#"<line x1="100" y1="{sep_y}" x2="300" y2="{sep_y}" stroke="{separator_color}" stroke-width="1"/>
    <image x="165" y="{icon_y}" width="20" height="20" href="{uri}" preserveAspectRatio="xMidYMid meet"/>
    <text x="193" y="{text_y}" font-family="Inter, sans-serif" font-size="16" font-weight="bold" fill="{text_fill}">Blurb</text>
    {url_line}"#
            )
        }
        None => {
            format!(
                r#"<line x1="100" y1="{sep_y}" x2="300" y2="{sep_y}" stroke="{separator_color}" stroke-width="1"/>
    <text x="200" y="{text_y}" text-anchor="middle" font-family="Inter, sans-serif" font-size="16" font-weight="bold" fill="{text_fill}">Blurb</text>
    {url_line}"#
            )
        }
    }
}

fn render_svg_to_png(svg: &str) -> Result<Vec<u8>> {
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    if let Some(fonts_dir) = bundled_fonts_dir() {
        for name in [
            "Inter-Regular.ttf",
            "Inter-Bold.ttf",
            "SourceSerif4-Regular.ttf",
            "SourceSerif4-It.ttf",
        ] {
            let _ = opt.fontdb_mut().load_font_file(fonts_dir.join(name));
        }
    }
    let tree = resvg::usvg::Tree::from_str(svg, &opt)?;
    let size = tree.size();
    let mut pixmap = tiny_skia::Pixmap::new(
        size.width() as u32 * SCALE_FACTOR,
        size.height() as u32 * SCALE_FACTOR,
    )
    .unwrap();
    let scale = SCALE_FACTOR as f32;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap.encode_png()?)
}

fn build_book_svg(
    title: &str,
    author: &str,
    rating: Option<i32>,
    cover_source: Option<&str>,
) -> String {
    let icon_uri = load_app_icon();
    let mut y = 20.0;

    let cover_y = y;
    let cover_element = match cover_source.and_then(load_cover_image) {
        Some(data_uri) => format!(
            r#"<rect x="98" y="{shadow_y}" width="204" height="304" rx="4" fill="black" opacity="0.2" filter="url(#coverShadow)"/>
  <image x="100" y="{cover_y}" width="200" height="300" href="{data_uri}" preserveAspectRatio="xMidYMid meet"/>"#,
            shadow_y = cover_y + 3.0,
        ),
        None => cover_fallback(title, cover_y),
    };
    y += 300.0 + 24.0;

    let title_lines = word_wrap(title, 25);
    let title_line_height = 26.0;
    let title_base_y = y;
    let title_element = if title_lines.len() == 1 {
        format!(
            r##"<text x="200" y="{title_base_y}" text-anchor="middle" font-family="Inter, sans-serif" font-size="22" font-weight="bold" fill="#1a1a2e">{}</text>"##,
            escape_xml(&title_lines[0])
        )
    } else {
        let tspans: String = title_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let ty = title_base_y + (i as f64 * title_line_height);
                format!(r#"<tspan x="200" y="{ty}">{}</tspan>"#, escape_xml(line))
            })
            .collect::<Vec<_>>()
            .join("\n    ");
        format!(
            r##"<text text-anchor="middle" font-family="Inter, sans-serif" font-size="22" font-weight="bold" fill="#1a1a2e">
    {tspans}
  </text>"##
        )
    };
    y += (title_lines.len() as f64 - 1.0) * title_line_height + 26.0;

    let author_y = y;
    let author_escaped = escape_xml(author);
    y += 36.0;

    let rating_svg = rating
        .map(|r| {
            let ry = y;
            y += 30.0;
            render_stars(r, 200.0, ry)
        })
        .unwrap_or_default();

    let footer_y = y.max(500.0);
    let footer = branded_footer(icon_uri.as_deref(), false, footer_y);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{CARD_WIDTH}" height="{CARD_HEIGHT}">
  <defs>
    <clipPath id="cardClip">
      <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" rx="16"/>
    </clipPath>
    <filter id="coverShadow" x="-10%" y="-10%" width="130%" height="130%">
      <feGaussianBlur in="SourceAlpha" stdDeviation="6"/>
      <feOffset dy="3" result="shadow"/>
      <feMerge>
        <feMergeNode in="shadow"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    <filter id="cardShadow" x="-5%" y="-5%" width="110%" height="110%">
      <feDropShadow dx="0" dy="2" stdDeviation="8" flood-color="rgba(0,0,0,0.15)"/>
    </filter>
  </defs>
  <g clip-path="url(#cardClip)">
    <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" fill="#f5f0eb"/>
    {cover_element}
    {title_element}
    <text x="200" y="{author_y}" text-anchor="middle" font-family="Inter, sans-serif" font-size="16" fill="#6b7280">{author_escaped}</text>
    {rating_svg}
    {footer}
  </g>
  <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" rx="16" fill="none" stroke="rgba(0,0,0,0.06)" stroke-width="1" filter="url(#cardShadow)"/>
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
    let icon_uri = load_app_icon();
    let lines = clamp_lines(word_wrap(quote, 35), 12);
    let line_height = 28.0;

    let mut y = 60.0;

    let open_quote_y = y;
    y += 40.0;

    let start_y = y;
    let tspans: String = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let ty = start_y + (i as f64 * line_height);
            format!(r#"<tspan x="200" y="{ty}">{}</tspan>"#, escape_xml(line))
        })
        .collect::<Vec<_>>()
        .join("\n    ");
    y += lines.len() as f64 * line_height + 10.0;

    let close_quote_y = y;
    y += 40.0;

    let title_y = y;
    y += 28.0;

    let author_y = y;
    y += 30.0;

    let title_escaped = escape_xml(book_title);
    let author_escaped = escape_xml(author);

    let rating_svg = rating
        .map(|r| {
            let ry = y;
            y += 30.0;
            render_stars(r, 200.0, ry)
        })
        .unwrap_or_default();

    let footer_y = y.max(500.0);
    let footer = branded_footer(icon_uri.as_deref(), true, footer_y);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{CARD_WIDTH}" height="{CARD_HEIGHT}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#0f1b3d"/>
      <stop offset="45%" stop-color="#162244"/>
      <stop offset="100%" stop-color="#1a1a2e"/>
    </linearGradient>
    <clipPath id="cardClip">
      <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" rx="16"/>
    </clipPath>
  </defs>
  <g clip-path="url(#cardClip)">
    <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" fill="url(#bg)"/>
    <text x="40" y="{open_quote_y}" font-family="Source Serif 4, serif" font-size="60" fill="#6b7fa3" opacity="0.5">&#x201C;</text>
    <text text-anchor="middle" font-family="Source Serif 4, serif" font-size="18" fill="#e2e8f0" font-style="italic">
      {tspans}
    </text>
    <text x="360" y="{close_quote_y}" font-family="Source Serif 4, serif" font-size="60" fill="#6b7fa3" opacity="0.5">&#x201D;</text>
    <text x="200" y="{title_y}" text-anchor="middle" font-family="Inter, sans-serif" font-size="16" font-weight="bold" fill="white">{title_escaped}</text>
    <text x="200" y="{author_y}" text-anchor="middle" font-family="Inter, sans-serif" font-size="14" fill="#a0aec0">{author_escaped}</text>
    {rating_svg}
    {footer}
  </g>
  <rect width="{CARD_WIDTH}" height="{CARD_HEIGHT}" rx="16" fill="none" stroke="rgba(255,255,255,0.08)" stroke-width="1"/>
</svg>"##
    )
}

fn cover_fallback(title: &str, y: f64) -> String {
    let letter = title
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let text_y = y + 180.0;
    format!(
        r##"<rect x="100" y="{y}" width="200" height="300" rx="8" fill="#d6cfc7"/>
  <text x="200" y="{text_y}" text-anchor="middle" font-family="Inter, sans-serif" font-size="72" font-weight="bold" fill="#7a7068">{letter}</text>"##
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
