use std::collections::HashMap;

use crate::history::History;
use crate::node::{Fragment, Node};
use crate::position::resolve;
use crate::schema::{ContentExpr, MarkType, NodeType};
use crate::slice::Slice;
use crate::state::{EditorState, Selection};
use crate::step::{all_have_mark_in_range, Step};
use crate::transform::Transform;

/// Toggle a mark on the current selection.
/// If all text in the selection has the mark, remove it; otherwise add it.
pub fn toggle_mark(state: &EditorState, mark: MarkType) -> Option<Transform> {
    let from = state.selection.from();
    let to = state.selection.to();
    if from == to {
        return None;
    }

    let mut tr = Transform::new(state.doc.clone());
    if all_have_mark_in_range(&state.doc, from, to, &mark) {
        tr.add_step(Step::RemoveMark { from, to, mark }).ok()?;
    } else {
        tr.add_step(Step::AddMark { from, to, mark }).ok()?;
    }
    Some(tr)
}

/// Change the block type of the block containing the cursor.
pub fn set_block_type(
    state: &EditorState,
    target_type: NodeType,
    attrs: HashMap<String, String>,
) -> Option<Transform> {
    let resolved = resolve(&state.doc, state.selection.anchor)?;

    // Find the innermost textblock (paragraph, heading, codeblock)
    let tb_depth = find_textblock_depth_resolved(&resolved)?;
    if tb_depth == 0 {
        return None;
    }

    // Navigate to the textblock node
    let block = node_at_depth(&state.doc, &resolved, tb_depth)?;

    // Already the right type? Check attrs too.
    if block.node_type == target_type && block.attrs == attrs {
        return None;
    }

    // Compute block position: content_start - 1 = open token position
    let block_start = resolved.start_at(tb_depth)? - 1;
    let block_end = block_start + block.node_size();

    let new_block = Node {
        node_type: target_type,
        attrs,
        content: block.content.clone(),
        marks: Vec::new(),
        text: None,
    };

    let slice = Slice::new(Fragment::from_vec(vec![new_block]), 0, 0);
    let mut tr = Transform::new(state.doc.clone());
    tr.add_step(Step::Replace {
        from: block_start,
        to: block_end,
        slice,
    })
    .ok()?;
    Some(tr)
}

/// Wrap the block(s) at the selection in a container node.
pub fn wrap_in(
    state: &EditorState,
    wrapper_type: NodeType,
    attrs: HashMap<String, String>,
) -> Option<Transform> {
    let resolved = resolve(&state.doc, state.selection.anchor)?;

    // Find the block to wrap (innermost textblock or its parent's child)
    let (block, block_start, block_end) = find_wrappable_block(&state.doc, &resolved)?;

    // Build the wrapper structure
    let wrapped = match wrapper_type {
        NodeType::BulletList | NodeType::OrderedList => {
            // List > ListItem > block
            let list_item = Node::new(NodeType::ListItem, vec![block.clone()]);
            Node {
                node_type: wrapper_type,
                attrs,
                content: Fragment::from_vec(vec![list_item]),
                marks: Vec::new(),
                text: None,
            }
        }
        _ => Node {
            node_type: wrapper_type,
            attrs,
            content: Fragment::from_vec(vec![block.clone()]),
            marks: Vec::new(),
            text: None,
        },
    };

    let slice = Slice::new(Fragment::from_vec(vec![wrapped]), 0, 0);
    let mut tr = Transform::new(state.doc.clone());
    tr.add_step(Step::Replace {
        from: block_start,
        to: block_end,
        slice,
    })
    .ok()?;
    Some(tr)
}

/// Lift (unwrap) the block at the selection from its parent container.
pub fn lift(state: &EditorState) -> Option<Transform> {
    let resolved = resolve(&state.doc, state.selection.anchor)?;

    // Find a liftable container (blockquote, list item, etc.)
    let (container, container_start, container_end) =
        find_liftable_container(&state.doc, &resolved)?;

    // Replace the container with its children
    let children: Vec<Node> = container.content.children().to_vec();
    let slice = Slice::new(Fragment::from_vec(children), 0, 0);
    let mut tr = Transform::new(state.doc.clone());
    tr.add_step(Step::Replace {
        from: container_start,
        to: container_end,
        slice,
    })
    .ok()?;
    Some(tr)
}

/// Insert a node (e.g., HorizontalRule) at the current cursor position.
pub fn insert_node(state: &EditorState, node: Node) -> Option<Transform> {
    let pos = state.selection.from();
    let resolved = resolve(&state.doc, pos)?;

    // Find the block boundary to insert at
    // We insert after the current block
    let tb_depth = find_textblock_depth_resolved(&resolved).unwrap_or(1);
    if tb_depth == 0 {
        return None;
    }

    let block = node_at_depth(&state.doc, &resolved, tb_depth)?;
    let block_start = resolved.start_at(tb_depth)? - 1;
    let block_end = block_start + block.node_size();

    // Insert the node after the current block
    let slice = Slice::new(Fragment::from_vec(vec![block.clone(), node]), 0, 0);
    let mut tr = Transform::new(state.doc.clone());
    tr.add_step(Step::Replace {
        from: block_start,
        to: block_end,
        slice,
    })
    .ok()?;
    Some(tr)
}

/// Undo the last change. Returns (transform, new_selection) if possible.
pub fn undo(state: &EditorState, history: &mut History) -> Option<(Transform, Selection)> {
    let result = history.undo(&state.doc)?;
    let tr = result.ok()?;
    let new_sel = state.selection.map_through(&tr);
    Some((tr, new_sel))
}

/// Redo the last undone change. Returns (transform, new_selection) if possible.
pub fn redo(state: &EditorState, history: &mut History) -> Option<(Transform, Selection)> {
    let result = history.redo(&state.doc)?;
    let tr = result.ok()?;
    let new_sel = state.selection.map_through(&tr);
    Some((tr, new_sel))
}

// --- Helpers ---

use crate::position::ResolvedPos;

fn find_textblock_depth_resolved(resolved: &ResolvedPos) -> Option<usize> {
    for d in (0..=resolved.depth()).rev() {
        if let Some(nt) = resolved.node_type_at(d) {
            if nt.content_expr() == ContentExpr::Inline && nt != NodeType::Text {
                return Some(d);
            }
        }
    }
    None
}

fn node_at_depth<'a>(doc: &'a Node, resolved: &ResolvedPos, depth: usize) -> Option<&'a Node> {
    let mut current = doc;
    for d in 0..depth {
        let index = resolved.index_at(d)?;
        current = current.content.child(index)?;
    }
    Some(current)
}

fn find_wrappable_block<'a>(
    doc: &'a Node,
    resolved: &ResolvedPos,
) -> Option<(&'a Node, usize, usize)> {
    let tb_depth = find_textblock_depth_resolved(resolved)?;
    if tb_depth == 0 {
        return None;
    }
    let block = node_at_depth(doc, resolved, tb_depth)?;
    let block_start = resolved.start_at(tb_depth)? - 1;
    let block_end = block_start + block.node_size();
    Some((block, block_start, block_end))
}

fn find_liftable_container<'a>(
    doc: &'a Node,
    resolved: &ResolvedPos,
) -> Option<(&'a Node, usize, usize)> {
    // Walk up from the deepest level to find a container we can lift from
    for d in (1..=resolved.depth()).rev() {
        let nt = resolved.node_type_at(d)?;
        if matches!(
            nt,
            NodeType::Blockquote
                | NodeType::BulletList
                | NodeType::OrderedList
                | NodeType::ListItem
        ) {
            let container = node_at_depth(doc, resolved, d)?;
            let container_start = resolved.start_at(d)? - 1;
            let container_end = container_start + container.node_size();
            return Some((container, container_start, container_end));
        }
    }
    None
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
