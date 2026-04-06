use crate::node::Node;
use crate::transform::Transform;

/// A text selection defined by anchor and head positions.
#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    /// The fixed end of the selection (where the user started selecting).
    pub anchor: usize,
    /// The moving end of the selection (where the cursor is).
    pub head: usize,
}

impl Selection {
    pub fn cursor(pos: usize) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub fn from(&self) -> usize {
        self.anchor.min(self.head)
    }

    pub fn to(&self) -> usize {
        self.anchor.max(self.head)
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }

    /// Map this selection through a transform's position changes.
    pub fn map_through(&self, tr: &Transform) -> Self {
        Self {
            anchor: tr.map_pos(self.anchor, -1),
            head: tr.map_pos(self.head, 1),
        }
    }
}

/// The editor state: a document plus selection.
#[derive(Debug, Clone)]
pub struct EditorState {
    pub doc: Node,
    pub selection: Selection,
}

impl EditorState {
    pub fn new(doc: Node, selection: Selection) -> Self {
        Self { doc, selection }
    }

    /// Create a state with the cursor at position 0.
    pub fn from_doc(doc: Node) -> Self {
        Self {
            doc,
            selection: Selection::cursor(0),
        }
    }

    /// Apply a transform to produce a new state.
    /// The selection is mapped through the transform's position changes.
    pub fn apply_transform(&self, tr: &Transform) -> Self {
        Self {
            doc: tr.doc().clone(),
            selection: self.selection.map_through(tr),
        }
    }

    /// Apply a transform with an explicit new selection.
    pub fn apply_transform_with_selection(&self, tr: &Transform, selection: Selection) -> Self {
        Self {
            doc: tr.doc().clone(),
            selection,
        }
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
