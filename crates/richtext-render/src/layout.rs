use parley::layout::{Layout, PositionedLayoutItem};
use parley::style::{FontFamily, FontStyle, FontWeight, StyleProperty};
use parley::{FontContext, LayoutContext};
use richtext_core::node::Node;
use richtext_core::schema::{MarkType, NodeType};

/// A laid-out block ready for painting.
pub struct LayoutBlock {
    pub layout: Layout<Color>,
    pub x: f32,
    pub y: f32,
    pub bg: Option<Color>,
    pub is_hr: bool,
    pub bullet: Option<String>,
    pub bullet_x: f32,
    /// Flat document position where this block's inline content starts.
    pub doc_content_start: usize,
    /// Length of the concatenated text in this block (characters/bytes for ASCII).
    pub text_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn to_u32(self) -> u32 {
        u32::from_be_bytes([0, self.r, self.g, self.b])
    }
}

const BODY_SIZE: f32 = 16.0;
const H1_SIZE: f32 = 32.0;
const H2_SIZE: f32 = 24.0;
const H3_SIZE: f32 = 20.0;
const LINE_GAP: f32 = 8.0;
const LIST_INDENT: f32 = 24.0;
const BLOCKQUOTE_INDENT: f32 = 20.0;
pub const CODE_BLOCK_PAD: f32 = 12.0;
const HR_HEIGHT: f32 = 20.0;

const TEXT_COLOR: Color = Color::rgb(30, 30, 30);
const CODE_BG: Color = Color::rgb(240, 240, 240);

struct LayoutState<'a> {
    font_cx: &'a mut FontContext,
    layout_cx: &'a mut LayoutContext<Color>,
    max_width: f32,
    scale: f32,
    blocks: Vec<LayoutBlock>,
    y: f32,
}

/// Lay out an entire document into positioned blocks.
pub fn layout_document(
    doc: &Node,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext<Color>,
    max_width: f32,
) -> Vec<LayoutBlock> {
    layout_document_scaled(doc, font_cx, layout_cx, max_width, 1.0)
}

/// Lay out a document with a display scale factor (e.g. 2.0 for Retina).
/// `max_width` should be in logical points. Resulting coordinates are in physical pixels.
pub fn layout_document_scaled(
    doc: &Node,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext<Color>,
    max_width: f32,
    scale: f32,
) -> Vec<LayoutBlock> {
    let mut state = LayoutState {
        font_cx,
        layout_cx,
        max_width,
        scale,
        blocks: Vec::new(),
        y: 10.0 * scale,
    };
    layout_blocks(&mut state, doc, 0.0, 0, 0);
    state.blocks
}

fn layout_blocks(
    state: &mut LayoutState<'_>,
    node: &Node,
    x_offset: f32,
    list_index: usize,
    doc_pos: usize,
) {
    match node.node_type {
        NodeType::Doc => {
            let mut pos = doc_pos;
            for child in node.content.children() {
                layout_blocks(state, child, x_offset, 0, pos);
                pos += child.node_size();
            }
        }
        NodeType::Paragraph => {
            let content_start = doc_pos + 1;
            let text_len = node.content.size();
            let s = state.scale;
            let available = state.max_width - x_offset;
            let layout = build_inline_layout(
                node,
                state.font_cx,
                state.layout_cx,
                available,
                BODY_SIZE * s,
                s,
            );
            state.blocks.push(LayoutBlock {
                layout,
                x: x_offset,
                y: state.y,
                bg: None,
                is_hr: false,
                bullet: None,
                bullet_x: 0.0,
                doc_content_start: content_start,
                text_len,
            });
            state.y += state.blocks.last().unwrap().layout.height() + LINE_GAP * s;
        }
        NodeType::Heading => {
            let content_start = doc_pos + 1;
            let text_len = node.content.size();
            let s = state.scale;
            let level: u8 = node
                .attrs
                .get("level")
                .and_then(|l| l.parse().ok())
                .unwrap_or(1);
            let font_size = match level {
                1 => H1_SIZE,
                2 => H2_SIZE,
                _ => H3_SIZE,
            };
            let available = state.max_width - x_offset;
            let layout = build_inline_layout(
                node,
                state.font_cx,
                state.layout_cx,
                available,
                font_size * s,
                s,
            );
            state.y += font_size * s * 0.3;
            state.blocks.push(LayoutBlock {
                layout,
                x: x_offset,
                y: state.y,
                bg: None,
                is_hr: false,
                bullet: None,
                bullet_x: 0.0,
                doc_content_start: content_start,
                text_len,
            });
            state.y += state.blocks.last().unwrap().layout.height() + LINE_GAP * s;
        }
        NodeType::BulletList => {
            let s = state.scale;
            let mut pos = doc_pos + 1;
            for (i, child) in node.content.children().iter().enumerate() {
                layout_blocks(state, child, x_offset + LIST_INDENT * s, i, pos);
                pos += child.node_size();
            }
        }
        NodeType::OrderedList => {
            let s = state.scale;
            let mut pos = doc_pos + 1;
            for (i, child) in node.content.children().iter().enumerate() {
                layout_blocks(state, child, x_offset + LIST_INDENT * s, i + 1, pos);
                pos += child.node_size();
            }
        }
        NodeType::ListItem => {
            let s = state.scale;
            let bullet_text = if list_index > 0 {
                format!("{}.", list_index)
            } else {
                "\u{2022}".to_string()
            };
            let mut first = true;
            let mut pos = doc_pos + 1;
            for child in node.content.children() {
                layout_blocks(state, child, x_offset, 0, pos);
                if first {
                    if let Some(block) = state.blocks.last_mut() {
                        block.bullet = Some(bullet_text.clone());
                        block.bullet_x = x_offset - LIST_INDENT * s + 4.0 * s;
                    }
                    first = false;
                }
                pos += child.node_size();
            }
        }
        NodeType::Blockquote => {
            let s = state.scale;
            let mut pos = doc_pos + 1;
            for child in node.content.children() {
                layout_blocks(state, child, x_offset + BLOCKQUOTE_INDENT * s, 0, pos);
                pos += child.node_size();
            }
        }
        NodeType::CodeBlock => {
            let content_start = doc_pos + 1;
            let text_len = node.content.size();
            let s = state.scale;
            let pad = CODE_BLOCK_PAD * s;
            let available = state.max_width - x_offset - pad * 2.0;
            let text = collect_text(node);
            let layout = build_plain_layout(
                state.font_cx,
                state.layout_cx,
                &text,
                available,
                BODY_SIZE * s,
                true,
                s,
            );
            let block_y = state.y;
            state.blocks.push(LayoutBlock {
                layout,
                x: x_offset + pad,
                y: block_y + pad,
                bg: Some(CODE_BG),
                is_hr: false,
                bullet: None,
                bullet_x: 0.0,
                doc_content_start: content_start,
                text_len,
            });
            let block_height = state.blocks.last().unwrap().layout.height() + pad * 2.0;
            state.y += block_height + LINE_GAP * s;
        }
        NodeType::HorizontalRule => {
            let s = state.scale;
            let layout = build_plain_layout(
                state.font_cx,
                state.layout_cx,
                " ",
                state.max_width,
                1.0,
                false,
                s,
            );
            state.blocks.push(LayoutBlock {
                layout,
                x: x_offset,
                y: state.y,
                bg: None,
                is_hr: true,
                bullet: None,
                bullet_x: 0.0,
                doc_content_start: doc_pos,
                text_len: 0,
            });
            state.y += HR_HEIGHT * s;
        }
        NodeType::Text => {}
    }
}

fn collect_text(node: &Node) -> String {
    let mut out = String::new();
    collect_text_inner(node, &mut out);
    out
}

fn collect_text_inner(node: &Node, out: &mut String) {
    if let Some(ref t) = node.text {
        out.push_str(t);
    }
    for child in node.content.children() {
        collect_text_inner(child, out);
    }
}

/// Build a Parley layout for a node containing inline content (paragraphs, headings).
fn build_inline_layout(
    node: &Node,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext<Color>,
    max_width: f32,
    base_size: f32,
    scale: f32,
) -> Layout<Color> {
    let text = collect_text(node);

    let mut builder = layout_cx.ranged_builder(font_cx, &text, scale, true);
    builder.push_default(StyleProperty::FontSize(base_size));
    builder.push_default(StyleProperty::Brush(TEXT_COLOR));
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
        std::borrow::Cow::Borrowed("system-ui"),
    )));

    // Walk inline children and apply mark-based styles to ranges
    let mut offset = 0usize;
    apply_inline_styles(node, &mut builder, &mut offset, base_size);

    let mut layout = builder.build(&text);
    layout.break_all_lines(Some(max_width));
    layout.align(
        Some(max_width),
        parley::layout::Alignment::Start,
        parley::AlignmentOptions::default(),
    );
    layout
}

fn apply_inline_styles(
    node: &Node,
    builder: &mut parley::RangedBuilder<'_, Color>,
    offset: &mut usize,
    base_size: f32,
) {
    if node.is_text() {
        let len = node.text.as_ref().map_or(0, |t| t.len());
        let range = *offset..*offset + len;
        for mark in &node.marks {
            match mark {
                MarkType::Bold => {
                    builder.push(StyleProperty::FontWeight(FontWeight::BOLD), range.clone());
                }
                MarkType::Italic => {
                    builder.push(StyleProperty::FontStyle(FontStyle::Italic), range.clone());
                }
                MarkType::Strike => {
                    builder.push(StyleProperty::Strikethrough(true), range.clone());
                }
                MarkType::Code => {
                    builder.push(
                        StyleProperty::FontFamily(FontFamily::Source(std::borrow::Cow::Borrowed(
                            "monospace",
                        ))),
                        range.clone(),
                    );
                    builder.push(StyleProperty::FontSize(base_size * 0.9), range.clone());
                }
                MarkType::Link { .. } => {
                    builder.push(StyleProperty::Brush(Color::rgb(30, 80, 200)), range.clone());
                }
            }
        }
        *offset += len;
    }
    for child in node.content.children() {
        apply_inline_styles(child, builder, offset, base_size);
    }
}

fn build_plain_layout(
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext<Color>,
    text: &str,
    max_width: f32,
    font_size: f32,
    monospace: bool,
    scale: f32,
) -> Layout<Color> {
    let mut builder = layout_cx.ranged_builder(font_cx, text, scale, true);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(StyleProperty::Brush(TEXT_COLOR));
    let family = if monospace { "monospace" } else { "system-ui" };
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
        std::borrow::Cow::Borrowed(family),
    )));

    let mut layout = builder.build(text);
    layout.break_all_lines(Some(max_width));
    layout.align(
        Some(max_width),
        parley::layout::Alignment::Start,
        parley::AlignmentOptions::default(),
    );
    layout
}

// --- Position mapping helpers (ASCII-only: 1 glyph = 1 byte) ---

struct LineGlyphInfo {
    char_start: usize,
    char_end: usize,
    x_positions: Vec<f32>,
    baseline: f32,
    font_size: f32,
}

fn collect_line_info(layout: &Layout<Color>) -> Vec<LineGlyphInfo> {
    let mut lines = Vec::new();
    let mut char_count = 0;

    for line in layout.lines() {
        let char_start = char_count;
        let mut x_positions: Vec<f32> = Vec::new();
        let mut baseline = 0.0f32;
        let mut font_size = BODY_SIZE;
        let mut last_x = 0.0f32;

        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(gr) = item {
                baseline = gr.baseline();
                font_size = gr.run().font_size();
                let mut x = gr.offset();
                for glyph in gr.glyphs() {
                    x_positions.push(x);
                    x += glyph.advance;
                    char_count += 1;
                }
                last_x = x;
            }
        }
        x_positions.push(last_x);

        lines.push(LineGlyphInfo {
            char_start,
            char_end: char_count,
            x_positions,
            baseline,
            font_size,
        });
    }

    lines
}

/// Find which block contains a document position.
/// Returns (block_index, text_offset_within_block).
pub fn doc_pos_to_block(blocks: &[LayoutBlock], doc_pos: usize) -> Option<(usize, usize)> {
    for (i, block) in blocks.iter().enumerate() {
        if block.is_hr || block.text_len == 0 && doc_pos != block.doc_content_start {
            continue;
        }
        let start = block.doc_content_start;
        let end = start + block.text_len;
        if doc_pos >= start && doc_pos <= end {
            return Some((i, doc_pos - start));
        }
    }
    None
}

/// Compute caret pixel coordinates for a text offset within a block.
/// Returns (x, y_top, height) in absolute coordinates.
pub fn caret_coords(block: &LayoutBlock, text_offset: usize) -> (f32, f32, f32) {
    let lines = collect_line_info(&block.layout);
    if lines.is_empty() {
        return (block.x, block.y, 20.0);
    }

    for line in &lines {
        if text_offset <= line.char_end {
            let local_idx = text_offset.saturating_sub(line.char_start);
            let x = line
                .x_positions
                .get(local_idx)
                .copied()
                .unwrap_or_else(|| *line.x_positions.last().unwrap_or(&0.0));
            let height = line.font_size * 1.3;
            let y_top = line.baseline - line.font_size;
            return (block.x + x, block.y + y_top, height);
        }
    }

    // Past end — use last line
    let last = lines.last().unwrap();
    let x = *last.x_positions.last().unwrap_or(&0.0);
    let height = last.font_size * 1.3;
    let y_top = last.baseline - last.font_size;
    (block.x + x, block.y + y_top, height)
}

/// Hit-test: given absolute pixel coordinates, find the closest document position.
pub fn hit_test(blocks: &[LayoutBlock], x: f32, y: f32) -> usize {
    // Find closest text block by y
    let mut best_idx = None;
    let mut best_dist = f32::MAX;

    for (i, block) in blocks.iter().enumerate() {
        if block.is_hr || block.text_len == 0 {
            continue;
        }
        let center_y = block.y + block.layout.height() / 2.0;
        let dist = (y - center_y).abs();
        if dist < best_dist {
            best_dist = dist;
            best_idx = Some(i);
        }
    }

    let Some(idx) = best_idx else {
        return 0;
    };

    let block = &blocks[idx];
    let local_x = x - block.x;
    let local_y = y - block.y;
    let lines = collect_line_info(&block.layout);

    // Find closest line by y
    let mut best_line = 0;
    let mut best_line_dist = f32::MAX;
    for (li, line) in lines.iter().enumerate() {
        let center = line.baseline - line.font_size * 0.3;
        let dist = (local_y - center).abs();
        if dist < best_line_dist {
            best_line_dist = dist;
            best_line = li;
        }
    }

    if let Some(line) = lines.get(best_line) {
        // Find closest x position on this line
        let mut best_char = 0;
        let mut best_x_dist = f32::MAX;
        for (j, &gx) in line.x_positions.iter().enumerate() {
            let dist = (gx - local_x).abs();
            if dist < best_x_dist {
                best_x_dist = dist;
                best_char = j;
            }
        }
        let text_offset = (line.char_start + best_char).min(block.text_len);
        block.doc_content_start + text_offset
    } else {
        block.doc_content_start
    }
}

/// Compute selection highlight rectangles between two text offsets within a block.
/// Returns Vec<(x, y, width, height)> in absolute coordinates.
pub fn selection_rects(
    block: &LayoutBlock,
    from_offset: usize,
    to_offset: usize,
) -> Vec<(f32, f32, f32, f32)> {
    let from = from_offset.min(to_offset);
    let to = from_offset.max(to_offset);
    let lines = collect_line_info(&block.layout);
    let mut rects = Vec::new();

    for line in &lines {
        if from >= line.char_end || to <= line.char_start {
            continue;
        }
        let sel_start = from.max(line.char_start) - line.char_start;
        let sel_end = to.min(line.char_end) - line.char_start;

        let x1 = line.x_positions.get(sel_start).copied().unwrap_or(0.0);
        let x2 = line
            .x_positions
            .get(sel_end)
            .copied()
            .unwrap_or_else(|| *line.x_positions.last().unwrap_or(&0.0));

        let height = line.font_size * 1.3;
        let y_top = line.baseline - line.font_size;

        if x2 > x1 {
            rects.push((block.x + x1, block.y + y_top, x2 - x1, height));
        }
    }

    rects
}

/// Next valid text position after `doc_pos`, crossing block boundaries.
pub fn next_text_pos(blocks: &[LayoutBlock], doc_pos: usize) -> Option<usize> {
    for block in blocks {
        if block.is_hr || block.text_len == 0 {
            continue;
        }
        let start = block.doc_content_start;
        let end = start + block.text_len;
        if doc_pos < end {
            let next = if doc_pos < start { start } else { doc_pos + 1 };
            if next <= end {
                return Some(next);
            }
        }
    }
    None
}

/// Previous valid text position before `doc_pos`, crossing block boundaries.
pub fn prev_text_pos(blocks: &[LayoutBlock], doc_pos: usize) -> Option<usize> {
    for block in blocks.iter().rev() {
        if block.is_hr || block.text_len == 0 {
            continue;
        }
        let start = block.doc_content_start;
        let end = start + block.text_len;
        if doc_pos > start {
            let prev = if doc_pos > end { end } else { doc_pos - 1 };
            if prev >= start {
                return Some(prev);
            }
        }
    }
    None
}
