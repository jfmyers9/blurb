use super::*;
use crate::node::Node;
use crate::schema::{MarkType, NodeType};
use crate::step::Step;
use crate::transform::Transform;

fn doc_with_paragraph(text: &str) -> Node {
    Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::text(text, vec![])],
        )],
    )
}

#[test]
fn selection_cursor() {
    let sel = Selection::cursor(5);
    assert_eq!(sel.anchor, 5);
    assert_eq!(sel.head, 5);
    assert!(sel.is_collapsed());
    assert_eq!(sel.from(), 5);
    assert_eq!(sel.to(), 5);
}

#[test]
fn selection_range() {
    let sel = Selection::new(3, 8);
    assert!(!sel.is_collapsed());
    assert_eq!(sel.from(), 3);
    assert_eq!(sel.to(), 8);

    // Reversed selection
    let sel2 = Selection::new(8, 3);
    assert_eq!(sel2.from(), 3);
    assert_eq!(sel2.to(), 8);
}

#[test]
fn state_from_doc() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::from_doc(doc.clone());
    assert_eq!(state.doc, doc);
    assert_eq!(state.selection, Selection::cursor(0));
}

#[test]
fn apply_transform_updates_doc() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc.clone(), Selection::new(1, 6));

    let mut tr = Transform::new(doc);
    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();

    let new_state = state.apply_transform(&tr);
    let para = new_state.doc.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));
}

#[test]
fn apply_transform_maps_selection() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc.clone(), Selection::new(1, 6));

    let mut tr = Transform::new(doc);
    // Mark step doesn't change positions
    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();

    let new_state = state.apply_transform(&tr);
    assert_eq!(new_state.selection, Selection::new(1, 6));
}

#[test]
fn apply_transform_with_explicit_selection() {
    let doc = doc_with_paragraph("hello");
    let state = EditorState::new(doc.clone(), Selection::cursor(1));

    let mut tr = Transform::new(doc);
    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();

    let new_state = state.apply_transform_with_selection(&tr, Selection::cursor(3));
    assert_eq!(new_state.selection, Selection::cursor(3));
}
