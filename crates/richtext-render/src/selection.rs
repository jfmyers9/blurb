use tiny_skia::{Color as SkiaColor, Paint, PixmapMut, Rect, Transform};

use crate::layout::{self, LayoutBlock};

/// Draw selection highlight rectangles between two document positions.
pub fn draw_selection(pixmap: &mut PixmapMut<'_>, blocks: &[LayoutBlock], from: usize, to: usize) {
    for block in blocks {
        if block.is_hr || block.text_len == 0 {
            continue;
        }
        let start = block.doc_content_start;
        let end = start + block.text_len;

        if from >= end || to <= start {
            continue;
        }

        let sel_from = from.max(start) - start;
        let sel_to = to.min(end) - start;
        let rects = layout::selection_rects(block, sel_from, sel_to);

        let mut paint = Paint::default();
        paint.set_color(SkiaColor::from_rgba8(68, 138, 255, 80));

        for (rx, ry, rw, rh) in rects {
            if let Some(rect) = Rect::from_xywh(rx, ry, rw, rh) {
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }
    }
}
