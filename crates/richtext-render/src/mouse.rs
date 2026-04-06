use crate::layout::{self, LayoutBlock};

/// Tracks mouse interaction state for click and drag selection.
pub struct MouseState {
    pub pressed: bool,
    pub drag_anchor: Option<usize>,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            pressed: false,
            drag_anchor: None,
        }
    }

    /// Handle mouse press. Returns the document position at the click point.
    pub fn press(&mut self, blocks: &[LayoutBlock], x: f32, y: f32) -> usize {
        let pos = layout::hit_test(blocks, x, y);
        self.pressed = true;
        self.drag_anchor = Some(pos);
        pos
    }

    /// Handle mouse release.
    pub fn release(&mut self) {
        self.pressed = false;
        self.drag_anchor = None;
    }

    /// Handle mouse drag. Returns (anchor, head) if actively dragging.
    pub fn drag(&self, blocks: &[LayoutBlock], x: f32, y: f32) -> Option<(usize, usize)> {
        if !self.pressed {
            return None;
        }
        let anchor = self.drag_anchor?;
        let head = layout::hit_test(blocks, x, y);
        Some((anchor, head))
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}
