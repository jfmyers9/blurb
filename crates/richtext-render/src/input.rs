use std::time::{SystemTime, UNIX_EPOCH};

use richtext_core::history::History;
use richtext_core::node::{Fragment, Node};
use richtext_core::slice::Slice;
use richtext_core::state::{EditorState, Selection};
use richtext_core::step::Step;
use richtext_core::transform::Transform;

use crate::layout::{self, LayoutBlock};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Insert text at the current selection, replacing any selected range.
pub fn insert_text(
    state: &mut EditorState,
    history: &mut History,
    blocks: &[LayoutBlock],
    text: &str,
) -> bool {
    let from = state.selection.from();
    let to = state.selection.to();

    if layout::doc_pos_to_block(blocks, from).is_none() {
        return false;
    }

    let text_node = Node::text(text, Vec::new());
    let slice = Slice::new(Fragment::from_vec(vec![text_node]), 0, 0);
    let mut tr = Transform::new(state.doc.clone());

    if tr.add_step(Step::Replace { from, to, slice }).is_ok() {
        let new_pos = from + text.len();
        history.record(&tr, now_ms());
        *state = state.apply_transform_with_selection(&tr, Selection::cursor(new_pos));
        true
    } else {
        false
    }
}

/// Delete one character backward (Backspace).
pub fn delete_backward(
    state: &mut EditorState,
    history: &mut History,
    blocks: &[LayoutBlock],
) -> bool {
    let from = state.selection.from();
    let to = state.selection.to();

    if from != to {
        return delete_range(state, history, from, to);
    }

    if let Some((_, text_offset)) = layout::doc_pos_to_block(blocks, from) {
        if text_offset == 0 {
            return false;
        }
        return delete_range(state, history, from - 1, from);
    }
    false
}

/// Delete one character forward (Delete key).
pub fn delete_forward(
    state: &mut EditorState,
    history: &mut History,
    blocks: &[LayoutBlock],
) -> bool {
    let from = state.selection.from();
    let to = state.selection.to();

    if from != to {
        return delete_range(state, history, from, to);
    }

    if let Some((block_idx, text_offset)) = layout::doc_pos_to_block(blocks, from) {
        let block = &blocks[block_idx];
        if text_offset >= block.text_len {
            return false;
        }
        return delete_range(state, history, from, from + 1);
    }
    false
}

fn delete_range(state: &mut EditorState, history: &mut History, from: usize, to: usize) -> bool {
    let mut tr = Transform::new(state.doc.clone());
    if tr
        .add_step(Step::Replace {
            from,
            to,
            slice: Slice::empty(),
        })
        .is_ok()
    {
        history.record(&tr, now_ms());
        *state = state.apply_transform_with_selection(&tr, Selection::cursor(from));
        true
    } else {
        false
    }
}

/// Move cursor left. If `extend` is true, extend the selection.
pub fn move_left(state: &mut EditorState, blocks: &[LayoutBlock], extend: bool) {
    let pos = state.selection.head;
    let new_pos = layout::prev_text_pos(blocks, pos).unwrap_or(pos);
    if extend {
        state.selection = Selection::new(state.selection.anchor, new_pos);
    } else if !state.selection.is_collapsed() {
        state.selection = Selection::cursor(state.selection.from());
    } else {
        state.selection = Selection::cursor(new_pos);
    }
}

/// Move cursor right. If `extend` is true, extend the selection.
pub fn move_right(state: &mut EditorState, blocks: &[LayoutBlock], extend: bool) {
    let pos = state.selection.head;
    let new_pos = layout::next_text_pos(blocks, pos).unwrap_or(pos);
    if extend {
        state.selection = Selection::new(state.selection.anchor, new_pos);
    } else if !state.selection.is_collapsed() {
        state.selection = Selection::cursor(state.selection.to());
    } else {
        state.selection = Selection::cursor(new_pos);
    }
}

/// Undo the last change.
pub fn undo(state: &mut EditorState, history: &mut History) -> bool {
    if let Some((tr, sel)) = richtext_core::commands::undo(state, history) {
        *state = state.apply_transform_with_selection(&tr, sel);
        true
    } else {
        false
    }
}

/// Redo the last undone change.
pub fn redo(state: &mut EditorState, history: &mut History) -> bool {
    if let Some((tr, sel)) = richtext_core::commands::redo(state, history) {
        *state = state.apply_transform_with_selection(&tr, sel);
        true
    } else {
        false
    }
}
