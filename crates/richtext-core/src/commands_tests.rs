use super::*;
use crate::history::History;
use crate::node::{Fragment, Node};
use crate::schema::{MarkType, NodeType};
use crate::state::{EditorState, Selection};

fn doc_with_paragraph(text: &str) -> Node {
    Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::text(text, vec![])],
        )],
    )
}

fn doc_two_paragraphs(t1: &str, t2: &str) -> Node {
    Node::new(
        NodeType::Doc,
        vec![
            Node::new(NodeType::Paragraph, vec![Node::text(t1, vec![])]),
            Node::new(NodeType::Paragraph, vec![Node::text(t2, vec![])]),
        ],
    )
}

#[test]
fn toggle_mark_adds_bold() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::new(1, 6));

    let tr = toggle_mark(&state, MarkType::Bold).unwrap();
    let new_doc = tr.doc();

    let para = new_doc.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));
}

#[test]
fn toggle_mark_removes_if_all_bold() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::text("hello", vec![MarkType::Bold])],
        )],
    );
    let state = EditorState::new(doc, Selection::new(1, 6));

    let tr = toggle_mark(&state, MarkType::Bold).unwrap();
    let new_doc = tr.doc();

    let para = new_doc.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(!text.marks.contains(&MarkType::Bold));
}

#[test]
fn toggle_mark_no_selection_returns_none() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::cursor(3));

    assert!(toggle_mark(&state, MarkType::Bold).is_none());
}

#[test]
fn set_block_type_paragraph_to_heading() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::cursor(3));

    let mut attrs = HashMap::new();
    attrs.insert("level".to_string(), "1".to_string());
    let tr = set_block_type(&state, NodeType::Heading, attrs).unwrap();
    let new_doc = tr.doc();

    let block = new_doc.content.child(0).unwrap();
    assert_eq!(block.node_type, NodeType::Heading);
    assert_eq!(block.attrs.get("level").unwrap(), "1");

    // Content should be preserved
    let text = block.content.child(0).unwrap();
    assert_eq!(text.text.as_deref(), Some("hello"));
}

#[test]
fn set_block_type_heading_to_paragraph() {
    let doc = Node::new(
        NodeType::Doc,
        vec![
            Node::new(NodeType::Heading, vec![Node::text("title", vec![])]).with_attr("level", "1"),
        ],
    );
    let state = EditorState::new(doc, Selection::cursor(3));

    let tr = set_block_type(&state, NodeType::Paragraph, HashMap::new()).unwrap();
    let new_doc = tr.doc();

    let block = new_doc.content.child(0).unwrap();
    assert_eq!(block.node_type, NodeType::Paragraph);
}

#[test]
fn set_block_type_noop_when_same() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::cursor(3));

    assert!(set_block_type(&state, NodeType::Paragraph, HashMap::new()).is_none());
}

#[test]
fn wrap_in_blockquote() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::cursor(3));

    let tr = wrap_in(&state, NodeType::Blockquote, HashMap::new()).unwrap();
    let new_doc = tr.doc();

    let wrapper = new_doc.content.child(0).unwrap();
    assert_eq!(wrapper.node_type, NodeType::Blockquote);

    let inner = wrapper.content.child(0).unwrap();
    assert_eq!(inner.node_type, NodeType::Paragraph);
    assert_eq!(
        inner.content.child(0).unwrap().text.as_deref(),
        Some("hello")
    );
}

#[test]
fn wrap_in_bullet_list() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::cursor(3));

    let tr = wrap_in(&state, NodeType::BulletList, HashMap::new()).unwrap();
    let new_doc = tr.doc();

    let list = new_doc.content.child(0).unwrap();
    assert_eq!(list.node_type, NodeType::BulletList);

    let item = list.content.child(0).unwrap();
    assert_eq!(item.node_type, NodeType::ListItem);

    let para = item.content.child(0).unwrap();
    assert_eq!(para.node_type, NodeType::Paragraph);
}

#[test]
fn lift_from_blockquote() {
    // Doc > Blockquote > Paragraph("hello")
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Blockquote,
            vec![Node::new(
                NodeType::Paragraph,
                vec![Node::text("hello", vec![])],
            )],
        )],
    );
    // Cursor at position 3 (inside the paragraph inside the blockquote)
    // Doc content: Blockquote(open=0, BQ content_start=1, P(open=1, P content_start=2, text=2..7, P close=7), BQ close=8)
    // Positions: 0=BQ open, 1=P open, 2..7=hello, 7=P close, 8=BQ close
    // Cursor at 4 = 'l' in hello
    let state = EditorState::new(doc, Selection::cursor(4));

    let tr = lift(&state).unwrap();
    let new_doc = tr.doc();

    // Should now be Doc > Paragraph("hello")
    let first = new_doc.content.child(0).unwrap();
    assert_eq!(first.node_type, NodeType::Paragraph);
    assert_eq!(
        first.content.child(0).unwrap().text.as_deref(),
        Some("hello")
    );
}

#[test]
fn insert_horizontal_rule() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::cursor(3));

    let hr = Node::new(NodeType::HorizontalRule, vec![]);
    let tr = insert_node(&state, hr).unwrap();
    let new_doc = tr.doc();

    assert_eq!(new_doc.content.child_count(), 2);
    assert_eq!(
        new_doc.content.child(0).unwrap().node_type,
        NodeType::Paragraph
    );
    assert_eq!(
        new_doc.content.child(1).unwrap().node_type,
        NodeType::HorizontalRule
    );
}

#[test]
fn undo_redo_via_commands() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc, Selection::new(1, 6));
    let mut history = History::new();

    // Apply bold
    let tr = toggle_mark(&state, MarkType::Bold).unwrap();
    history.record(&tr, 1000);
    let bolded_state = state.apply_transform(&tr);

    // Undo
    let (undo_tr, _sel) = undo(&bolded_state, &mut history).unwrap();
    let unbolded_state = bolded_state.apply_transform(&undo_tr);

    let para = unbolded_state.doc.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(!text.marks.contains(&MarkType::Bold));

    // Redo
    let (redo_tr, _sel) = redo(&unbolded_state, &mut history).unwrap();
    let rebolded_state = unbolded_state.apply_transform(&redo_tr);

    let para = rebolded_state.doc.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));
}
