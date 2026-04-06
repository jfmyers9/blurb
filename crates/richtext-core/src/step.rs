use std::collections::HashMap;

use crate::node::{Fragment, Node};
use crate::schema::{MarkType, NodeType};
use crate::slice::Slice;

/// Maps positions through a step's document changes.
#[derive(Debug, Clone)]
pub struct StepMap {
    /// Each entry: (start, old_size, new_size)
    ranges: Vec<(usize, usize, usize)>,
}

impl StepMap {
    pub fn empty() -> Self {
        Self { ranges: vec![] }
    }

    pub fn new(start: usize, old_size: usize, new_size: usize) -> Self {
        Self {
            ranges: vec![(start, old_size, new_size)],
        }
    }

    /// Map a position through this step's changes.
    /// Bias: -1 maps to the left of insertions, 1 maps to the right.
    pub fn map(&self, pos: usize, bias: i32) -> usize {
        let Some(&(start, old_size, new_size)) = self.ranges.first() else {
            return pos;
        };
        let end = start + old_size;
        if pos < start {
            return pos;
        }
        if pos > end || (pos == end && bias > 0) || (pos == start && bias < 0 && old_size > 0) {
            let diff = new_size as isize - old_size as isize;
            return (pos as isize + diff).max(0) as usize;
        }
        if old_size == 0 {
            return pos;
        }
        // Position is within the replaced range
        if bias <= 0 {
            start
        } else {
            start + new_size
        }
    }
}

/// A single document transformation step.
#[derive(Debug, Clone)]
pub enum Step {
    Replace {
        from: usize,
        to: usize,
        slice: Slice,
    },
    AddMark {
        from: usize,
        to: usize,
        mark: MarkType,
    },
    RemoveMark {
        from: usize,
        to: usize,
        mark: MarkType,
    },
}

#[derive(Debug, Clone)]
pub enum StepError {
    InvalidPosition(usize),
    InvalidContent(String),
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepError::InvalidPosition(pos) => write!(f, "invalid position: {pos}"),
            StepError::InvalidContent(msg) => write!(f, "invalid content: {msg}"),
        }
    }
}

impl std::error::Error for StepError {}

impl Step {
    /// Apply this step to a document, returning (new_doc, inverted_step).
    pub fn apply(&self, doc: &Node) -> Result<(Node, Step), StepError> {
        match self {
            Step::Replace { from, to, slice } => {
                let old_slice = cut_slice(doc, *from, *to);
                let new_doc = replace_range(doc, *from, *to, slice)?;
                let inverted = Step::Replace {
                    from: *from,
                    to: *from + slice.size(),
                    slice: old_slice,
                };
                Ok((new_doc, inverted))
            }
            Step::AddMark { from, to, mark } => {
                let new_doc = map_text_in_range(doc, 0, *from, *to, &|node| {
                    if node.marks.contains(mark) {
                        return node.clone();
                    }
                    let mut new_marks = node.marks.clone();
                    new_marks.push(mark.clone());
                    Node {
                        marks: new_marks,
                        ..node.clone()
                    }
                });
                let inverted = Step::RemoveMark {
                    from: *from,
                    to: *to,
                    mark: mark.clone(),
                };
                Ok((new_doc, inverted))
            }
            Step::RemoveMark { from, to, mark } => {
                let new_doc = map_text_in_range(doc, 0, *from, *to, &|node| {
                    if !node.marks.contains(mark) {
                        return node.clone();
                    }
                    let new_marks = node.marks.iter().filter(|m| *m != mark).cloned().collect();
                    Node {
                        marks: new_marks,
                        ..node.clone()
                    }
                });
                let inverted = Step::AddMark {
                    from: *from,
                    to: *to,
                    mark: mark.clone(),
                };
                Ok((new_doc, inverted))
            }
        }
    }

    /// Get the position mapping for this step.
    pub fn step_map(&self) -> StepMap {
        match self {
            Step::Replace { from, to, slice } => StepMap::new(*from, to - from, slice.size()),
            Step::AddMark { .. } | Step::RemoveMark { .. } => StepMap::empty(),
        }
    }
}

/// Walk the document tree and apply a function to text nodes within [from, to).
fn map_text_in_range(
    node: &Node,
    base: usize,
    from: usize,
    to: usize,
    f: &dyn Fn(&Node) -> Node,
) -> Node {
    if node.is_text() {
        let end = base + node.node_size();
        if base < to && end > from {
            return f(node);
        }
        return node.clone();
    }

    if node.node_type.is_leaf() {
        return node.clone();
    }

    let content_start = if node.node_type == NodeType::Doc {
        base
    } else {
        base + 1
    };
    let mut offset = content_start;
    let mut new_children = Vec::new();
    let mut changed = false;

    for child in node.content.children() {
        let child_end = offset + child.node_size();
        let new_child = if offset < to && child_end > from {
            let mapped = map_text_in_range(child, offset, from, to, f);
            if mapped != *child {
                changed = true;
            }
            mapped
        } else {
            child.clone()
        };
        new_children.push(new_child);
        offset = child_end;
    }

    if changed {
        Node {
            node_type: node.node_type,
            attrs: node.attrs.clone(),
            content: Fragment::from_vec(new_children),
            marks: node.marks.clone(),
            text: node.text.clone(),
        }
    } else {
        node.clone()
    }
}

/// Extract a slice of content from [from, to) in the document.
pub fn cut_slice(doc: &Node, from: usize, to: usize) -> Slice {
    if from == to {
        return Slice::empty();
    }
    let content = cut_fragment(&doc.content, 0, from, to);
    Slice::new(content, 0, 0)
}

fn cut_fragment(fragment: &Fragment, base: usize, from: usize, to: usize) -> Fragment {
    let mut result = Vec::new();
    let mut pos = base;

    for child in fragment.children() {
        let child_end = pos + child.node_size();

        if child_end <= from || pos >= to {
            pos = child_end;
            continue;
        }

        if pos >= from && child_end <= to {
            result.push(child.clone());
        } else if child.is_text() {
            let text = child.text.as_ref().unwrap();
            let start = from.saturating_sub(pos);
            let end = if to < child_end { to - pos } else { text.len() };
            if start < end {
                result.push(Node::text(&text[start..end], child.marks.clone()));
            }
        } else if !child.is_leaf() {
            let inner_base = pos + 1;
            let inner_from = from.max(inner_base);
            let inner_to = to.min(child_end - 1);
            let cut = cut_fragment(&child.content, inner_base, inner_from, inner_to);
            result.push(Node {
                node_type: child.node_type,
                attrs: child.attrs.clone(),
                content: cut,
                marks: child.marks.clone(),
                text: None,
            });
        }
        pos = child_end;
    }

    Fragment::from_vec(result)
}

/// Replace the range [from, to) in the document with the given slice content.
fn replace_range(doc: &Node, from: usize, to: usize, slice: &Slice) -> Result<Node, StepError> {
    let new_content = replace_in_fragment(&doc.content, 0, from, to, &slice.content)?;
    Ok(Node {
        node_type: doc.node_type,
        attrs: doc.attrs.clone(),
        content: new_content,
        marks: doc.marks.clone(),
        text: None,
    })
}

fn replace_in_fragment(
    fragment: &Fragment,
    base: usize,
    from: usize,
    to: usize,
    insert: &Fragment,
) -> Result<Fragment, StepError> {
    let mut result = Vec::new();
    let mut pos = base;
    let mut inserted = false;

    for child in fragment.children() {
        let child_end = pos + child.node_size();

        if child_end <= from {
            result.push(child.clone());
        } else if pos >= to {
            if !inserted {
                for n in insert.children() {
                    result.push(n.clone());
                }
                inserted = true;
            }
            result.push(child.clone());
        } else if pos >= from && child_end <= to {
            // Entirely within replacement range
            if !inserted {
                for n in insert.children() {
                    result.push(n.clone());
                }
                inserted = true;
            }
        } else if child.is_text() {
            let text = child.text.as_ref().unwrap();
            if pos < from {
                let keep = from - pos;
                result.push(Node::text(&text[..keep], child.marks.clone()));
            }
            if !inserted {
                for n in insert.children() {
                    result.push(n.clone());
                }
                inserted = true;
            }
            if child_end > to {
                let skip = to - pos;
                result.push(Node::text(&text[skip..], child.marks.clone()));
            }
        } else if !child.is_leaf() {
            let inner_base = pos + 1;

            if pos < from && child_end > to {
                // Both from and to within this child — recurse
                let new_content =
                    replace_in_fragment(&child.content, inner_base, from, to, insert)?;
                result.push(Node {
                    node_type: child.node_type,
                    attrs: child.attrs.clone(),
                    content: new_content,
                    marks: child.marks.clone(),
                    text: None,
                });
                inserted = true;
            } else if pos < from {
                // from within this child — keep left part
                let inner_from = from.max(inner_base);
                let left = cut_fragment(&child.content, inner_base, inner_base, inner_from);
                result.push(Node {
                    node_type: child.node_type,
                    attrs: child.attrs.clone(),
                    content: left,
                    marks: child.marks.clone(),
                    text: None,
                });
                if !inserted {
                    for n in insert.children() {
                        result.push(n.clone());
                    }
                    inserted = true;
                }
            } else {
                // to within this child — keep right part
                if !inserted {
                    for n in insert.children() {
                        result.push(n.clone());
                    }
                    inserted = true;
                }
                let inner_to = to.min(child_end - 1);
                let right = cut_fragment(
                    &child.content,
                    inner_base,
                    inner_to,
                    inner_base + child.content.size(),
                );
                result.push(Node {
                    node_type: child.node_type,
                    attrs: child.attrs.clone(),
                    content: right,
                    marks: child.marks.clone(),
                    text: None,
                });
            }
        }
        pos = child_end;
    }

    if !inserted {
        for n in insert.children() {
            result.push(n.clone());
        }
    }

    Ok(Fragment::from_vec(result))
}

/// Helper: build a node with new content but same type/attrs.
pub fn node_with_content(node: &Node, content: Fragment) -> Node {
    Node {
        node_type: node.node_type,
        attrs: node.attrs.clone(),
        content,
        marks: node.marks.clone(),
        text: node.text.clone(),
    }
}

/// Helper: create a block node with the given type, attrs, and content.
pub fn make_block(node_type: NodeType, attrs: HashMap<String, String>, content: Vec<Node>) -> Node {
    Node {
        node_type,
        attrs,
        content: Fragment::from_vec(content),
        marks: Vec::new(),
        text: None,
    }
}

/// Check if any text node in [from, to) has the given mark.
pub fn has_mark_in_range(doc: &Node, from: usize, to: usize, mark: &MarkType) -> bool {
    let mut found = false;
    walk_text_in_range(doc, 0, from, to, &mut |node, _, _| {
        if node.marks.contains(mark) {
            found = true;
        }
    });
    found
}

/// Check if ALL text in [from, to) has the given mark.
pub fn all_have_mark_in_range(doc: &Node, from: usize, to: usize, mark: &MarkType) -> bool {
    let mut all = true;
    let mut any_text = false;
    walk_text_in_range(doc, 0, from, to, &mut |node, _, _| {
        any_text = true;
        if !node.marks.contains(mark) {
            all = false;
        }
    });
    any_text && all
}

fn walk_text_in_range(
    node: &Node,
    base: usize,
    from: usize,
    to: usize,
    f: &mut dyn FnMut(&Node, usize, usize),
) {
    if node.is_text() {
        let end = base + node.node_size();
        if base < to && end > from {
            f(node, base.max(from), end.min(to));
        }
        return;
    }
    if node.node_type.is_leaf() {
        return;
    }
    let content_start = if node.node_type == NodeType::Doc {
        base
    } else {
        base + 1
    };
    let mut offset = content_start;
    for child in node.content.children() {
        let child_end = offset + child.node_size();
        if offset < to && child_end > from {
            walk_text_in_range(child, offset, from, to, f);
        }
        offset = child_end;
    }
}

#[cfg(test)]
#[path = "step_tests.rs"]
mod tests;
