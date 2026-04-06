use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schema::{MarkType, NodeType};

/// An ordered list of child nodes with size tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fragment {
    children: Vec<Node>,
    /// Cached total size in flat position tokens.
    size: usize,
}

impl Fragment {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            size: 0,
        }
    }

    pub fn from_vec(children: Vec<Node>) -> Self {
        let size = children.iter().map(|c| c.node_size()).sum();
        Self { children, size }
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn child(&self, index: usize) -> Option<&Node> {
        self.children.get(index)
    }

    /// Total size in flat position tokens.
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Push a node and update size.
    pub fn push(&mut self, node: Node) {
        self.size += node.node_size();
        self.children.push(node);
    }

    /// Returns the child index and offset for a given flat position within this fragment.
    /// The offset is the position within the fragment (not the child).
    pub fn find_index(&self, pos: usize) -> Option<(usize, usize)> {
        let mut cur = 0;
        for (i, child) in self.children.iter().enumerate() {
            let end = cur + child.node_size();
            if pos < end {
                return Some((i, cur));
            }
            cur = end;
        }
        if pos == cur {
            Some((self.children.len(), cur))
        } else {
            None
        }
    }
}

/// A node in the document tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub node_type: NodeType,
    pub attrs: HashMap<String, String>,
    pub content: Fragment,
    /// Marks applied to this node (only meaningful for Text nodes).
    pub marks: Vec<MarkType>,
    /// Text content (only for Text nodes).
    pub text: Option<String>,
}

impl Node {
    /// Create a new non-text node.
    pub fn new(node_type: NodeType, children: Vec<Node>) -> Self {
        Self {
            node_type,
            attrs: HashMap::new(),
            content: Fragment::from_vec(children),
            marks: Vec::new(),
            text: None,
        }
    }

    /// Create a text node with optional marks.
    pub fn text(s: &str, marks: Vec<MarkType>) -> Self {
        Self {
            node_type: NodeType::Text,
            attrs: HashMap::new(),
            content: Fragment::empty(),
            marks,
            text: Some(s.to_string()),
        }
    }

    /// The size of this node in flat position tokens.
    /// Text nodes: length of text.
    /// Leaf nodes (no content): 1 token.
    /// Other nodes: open token + content size + close token.
    pub fn node_size(&self) -> usize {
        if self.node_type == NodeType::Text {
            self.text.as_ref().map_or(0, |t| t.len())
        } else if self.node_type.is_leaf() {
            1
        } else {
            // open + content + close
            self.content.size() + 2
        }
    }

    /// Whether this is a text node.
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::Text
    }

    /// Whether this node has no content.
    pub fn is_leaf(&self) -> bool {
        self.node_type.is_leaf()
    }

    /// The number of direct children.
    pub fn child_count(&self) -> usize {
        self.content.child_count()
    }

    /// Set an attribute.
    pub fn with_attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.insert(key.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
#[path = "node_tests.rs"]
mod tests;
