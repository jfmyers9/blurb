use super::*;
use crate::node::Node;
use crate::schema::{MarkType, NodeType};
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
fn empty_history() {
    let history = History::new();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.undo_depth(), 0);
    assert_eq!(history.redo_depth(), 0);
}

#[test]
fn undo_nothing_returns_none() {
    let doc = doc_with_paragraph("hello");
    let mut history = History::new();
    assert!(history.undo(&doc).is_none());
}

#[test]
fn record_and_undo() {
    let doc = doc_with_paragraph("hello");
    let mut history = History::new();

    // Apply bold
    let mut tr = Transform::new(doc.clone());
    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();

    let bolded_doc = tr.doc().clone();
    history.record(&tr, 1000);

    assert!(history.can_undo());
    assert!(!history.can_redo());

    // Undo
    let undo_result = history.undo(&bolded_doc).unwrap().unwrap();
    let restored = undo_result.doc();

    // Bold should be removed
    let para = restored.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(!text.marks.contains(&MarkType::Bold));

    assert!(!history.can_undo());
    assert!(history.can_redo());
}

#[test]
fn undo_then_redo() {
    let doc = doc_with_paragraph("hello");
    let mut history = History::new();

    let mut tr = Transform::new(doc.clone());
    tr.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();
    let bolded_doc = tr.doc().clone();
    history.record(&tr, 1000);

    // Undo
    let undo_tr = history.undo(&bolded_doc).unwrap().unwrap();
    let unbolded = undo_tr.doc().clone();

    // Redo
    let redo_tr = history.redo(&unbolded).unwrap().unwrap();
    let re_bolded = redo_tr.doc();

    let para = re_bolded.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));
}

#[test]
fn composition_groups_fast_changes() {
    let doc = doc_with_paragraph("hello");
    let mut history = History::new();

    // Two changes within 500ms should compose into one undo entry
    let mut tr1 = Transform::new(doc.clone());
    tr1.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();
    history.record(&tr1, 1000);

    let mut tr2 = Transform::new(tr1.doc().clone());
    tr2.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Italic,
    })
    .unwrap();
    history.record(&tr2, 1200); // 200ms later, within window

    assert_eq!(history.undo_depth(), 1); // Should be composed into one entry

    // Undo should remove both marks
    let undo_tr = history.undo(tr2.doc()).unwrap().unwrap();
    let restored = undo_tr.doc();
    let para = restored.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(!text.marks.contains(&MarkType::Bold));
    assert!(!text.marks.contains(&MarkType::Italic));
}

#[test]
fn no_composition_for_distant_changes() {
    let doc = doc_with_paragraph("hello");
    let mut history = History::new();

    let mut tr1 = Transform::new(doc.clone());
    tr1.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();
    history.record(&tr1, 1000);

    let mut tr2 = Transform::new(tr1.doc().clone());
    tr2.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Italic,
    })
    .unwrap();
    history.record(&tr2, 2000); // 1000ms later, outside window

    assert_eq!(history.undo_depth(), 2); // Should be separate entries
}

#[test]
fn new_change_clears_redo() {
    let doc = doc_with_paragraph("hello");
    let mut history = History::new();

    let mut tr1 = Transform::new(doc.clone());
    tr1.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    })
    .unwrap();
    history.record(&tr1, 1000);

    // Undo
    let undo_tr = history.undo(tr1.doc()).unwrap().unwrap();

    // New change should clear redo stack
    let mut tr2 = Transform::new(undo_tr.doc().clone());
    tr2.add_step(Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Italic,
    })
    .unwrap();
    history.record(&tr2, 3000);

    assert!(!history.can_redo());
}

#[test]
fn empty_transform_not_recorded() {
    let doc = doc_with_paragraph("hello");
    let mut history = History::new();

    let tr = Transform::new(doc);
    history.record(&tr, 1000);

    assert!(!history.can_undo());
}
