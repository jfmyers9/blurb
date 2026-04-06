use std::time::Instant;

use tiny_skia::{Color as SkiaColor, Paint, PixmapMut, Rect, Transform};

use crate::layout::{self, LayoutBlock};

pub struct CursorState {
    visible: bool,
    last_toggle: Instant,
    blink_ms: u128,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            visible: true,
            last_toggle: Instant::now(),
            blink_ms: 530,
        }
    }

    /// Returns true if visibility changed.
    pub fn tick(&mut self) -> bool {
        if self.last_toggle.elapsed().as_millis() >= self.blink_ms {
            self.visible = !self.visible;
            self.last_toggle = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.visible = true;
        self.last_toggle = Instant::now();
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn ms_until_toggle(&self) -> u64 {
        let elapsed = self.last_toggle.elapsed().as_millis() as u64;
        (self.blink_ms as u64).saturating_sub(elapsed)
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a blinking caret at the given document position.
pub fn draw_caret(pixmap: &mut PixmapMut<'_>, blocks: &[LayoutBlock], doc_pos: usize) {
    if let Some((block_idx, text_offset)) = layout::doc_pos_to_block(blocks, doc_pos) {
        let block = &blocks[block_idx];
        let (x, y, h) = layout::caret_coords(block, text_offset);
        if let Some(rect) = Rect::from_xywh(x, y, 2.0, h) {
            let mut paint = Paint::default();
            paint.set_color(SkiaColor::from_rgba8(0, 0, 0, 255));
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }
}
