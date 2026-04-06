use super::*;
use crate::node::{Fragment, Node};
use crate::schema::{MarkType, NodeType};
use crate::slice::Slice;
use crate::step::Step;

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
fn empty_transform() {
    let doc = doc_with_paragraph("hello");
    let tr = Transform::new(doc.clone());
    assert!(tr.is_empty());
    assert_eq!(tr.step_count(), 0);
    assert_eq!(tr.doc(), &doc);
}

#[test]
fn single_step_transform() {
    let doc = doc_with_paragraph("hello");
    let mut tr = Transform::new(doc);

    let inv = tr
        .add_step(Step::AddMark {
            from: 1,
            to: 6,
            mark: MarkType::Bold,
        })
        .unwrap();

    assert_eq!(tr.step_count(), 1);
    assert!(!tr.is_empty());

    let para = tr.doc().content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));

    matches!(inv, Step::RemoveMark { .. });
}

#[test]
fn multi_step_transform() {
    let doc = doc_with_paragraph("hello");
    let mut tr = Transform::new(doc);

    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();
    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Italic,
    })
    .unwrap();

    assert_eq!(tr.step_count(), 2);

    let para = tr.doc().content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));
    assert!(text.marks.contains(&MarkType::Italic));
}

#[test]
fn map_pos_through_replace() {
    let doc = doc_with_paragraph("hello");
    let mut tr = Transform::new(doc);

    // Delete "ell" (positions 2..5), replacing with nothing
    tr.add_step(Step::Replace {
        from: 2,
        to: 5,
        slice: Slice::empty(),
    })
    .unwrap();

    // Position after deleted range should shift left by 3
    assert_eq!(tr.map_pos(6, 1), 3);
    // Position before should stay
    assert_eq!(tr.map_pos(1, 1), 1);
}

#[test]
fn map_pos_through_mark_steps() {
    let doc = doc_with_paragraph("hello");
    let mut tr = Transform::new(doc);

    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();

    // Mark steps don't change positions
    assert_eq!(tr.map_pos(3, 1), 3);
}

#[test]
fn start_doc_preserved() {
    let doc = doc_with_paragraph("hello");
    let original = doc.clone();
    let mut tr = Transform::new(doc);

    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();

    assert_eq!(tr.start_doc(), &original);
    assert_ne!(tr.doc(), &original);
}
