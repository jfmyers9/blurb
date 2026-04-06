use serde::{Deserialize, Serialize};

use crate::node::Fragment;

/// A slice of a document, representing content extracted from a range.
/// `open_start` and `open_end` track how many levels of nodes are "open"
/// (not fully included) on each side, which is essential for paste operations
/// that need to fit content into the target context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slice {
    pub content: Fragment,
    pub open_start: usize,
    pub open_end: usize,
}

impl Slice {
    pub fn new(content: Fragment, open_start: usize, open_end: usize) -> Self {
        Self {
            content,
            open_start,
            open_end,
        }
    }

    pub fn empty() -> Self {
        Self {
            content: Fragment::empty(),
            open_start: 0,
            open_end: 0,
        }
    }

    /// Total size of the slice content.
    pub fn size(&self) -> usize {
        self.content.size()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

#[cfg(test)]
#[path = "slice_tests.rs"]
mod tests;
