use crate::node::Node;
use crate::schema::NodeType;

/// A resolved position in the document tree.
/// Contains the parent chain from the document root to the deepest node
/// containing this position.
#[derive(Debug, Clone)]
pub struct ResolvedPos {
    /// The flat position in the document.
    pub pos: usize,
    /// Stack of (node_type, offset_into_parent, index_in_parent) from root to deepest.
    /// Each entry: the node at that depth, the position where that node starts, and
    /// the index of the child at this position within that node.
    levels: Vec<Level>,
}

#[derive(Debug, Clone)]
struct Level {
    node_type: NodeType,
    /// Flat position of the start of this node's content (after its opening token).
    start: usize,
    /// Index of the relevant child at this level.
    index: usize,
}

impl ResolvedPos {
    /// The depth of the resolved position (0 = document root).
    pub fn depth(&self) -> usize {
        self.levels.len().saturating_sub(1)
    }

    /// The node type at the given depth.
    pub fn node_type_at(&self, depth: usize) -> Option<NodeType> {
        self.levels.get(depth).map(|l| l.node_type)
    }

    /// The start position of the node at the given depth (content start, after open token).
    pub fn start_at(&self, depth: usize) -> Option<usize> {
        self.levels.get(depth).map(|l| l.start)
    }

    /// The child index at the given depth.
    pub fn index_at(&self, depth: usize) -> Option<usize> {
        self.levels.get(depth).map(|l| l.index)
    }

    /// The parent node type (one level up from deepest).
    pub fn parent_type(&self) -> Option<NodeType> {
        if self.levels.len() >= 2 {
            Some(self.levels[self.levels.len() - 2].node_type)
        } else {
            None
        }
    }
}

/// Resolve a flat position into a ResolvedPos by walking the document tree.
/// Positions range from 0 to doc.content.size() (the doc's open/close tokens
/// are not part of the position space).
pub fn resolve(doc: &Node, pos: usize) -> Option<ResolvedPos> {
    let content_size = doc.content.size();
    if pos > content_size {
        return None;
    }

    let mut levels = Vec::new();
    // Doc is special: positions start at 0 within its content (no open token offset).
    resolve_at(doc, pos, 0, &mut levels);

    Some(ResolvedPos { pos, levels })
}

fn resolve_at(node: &Node, pos: usize, content_start: usize, levels: &mut Vec<Level>) {
    let inner_pos = pos - content_start;
    let mut offset = 0;

    for (i, child) in node.content.children().iter().enumerate() {
        let child_size = child.node_size();
        if inner_pos < offset + child_size {
            levels.push(Level {
                node_type: node.node_type,
                start: content_start,
                index: i,
            });
            // If child is a non-leaf, non-text node, descend into it.
            // The child's content starts after its open token.
            if !child.is_text() && !child.is_leaf() {
                let child_content_start = content_start + offset + 1;
                if pos >= child_content_start {
                    resolve_at(child, pos, child_content_start, levels);
                }
            }
            return;
        }
        offset += child_size;
    }

    // Position is at or past the end of all children
    levels.push(Level {
        node_type: node.node_type,
        start: content_start,
        index: node.content.child_count(),
    });
}

#[cfg(test)]
#[path = "position_tests.rs"]
mod tests;
