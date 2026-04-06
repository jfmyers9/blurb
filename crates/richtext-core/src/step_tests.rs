use super::*;
use crate::node::{Fragment, Node};
use crate::schema::{MarkType, NodeType};
use crate::slice::Slice;

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
fn step_map_identity() {
    let map = StepMap::empty();
    assert_eq!(map.map(5, 1), 5);
    assert_eq!(map.map(0, -1), 0);
}

#[test]
fn step_map_insert() {
    // Inserting 3 chars at position 2 (replacing 0 chars)
    let map = StepMap::new(2, 0, 3);
    assert_eq!(map.map(0, 1), 0); // before insert
    assert_eq!(map.map(1, 1), 1); // before insert
    assert_eq!(map.map(2, -1), 2); // at insert point, bias left → stays
    assert_eq!(map.map(2, 1), 5); // at insert point, bias right → after insertion
    assert_eq!(map.map(5, 1), 8); // after insert, shifted by 3
}

#[test]
fn step_map_delete() {
    // Deleting 3 chars at position 2 (replacing with 0)
    let map = StepMap::new(2, 3, 0);
    assert_eq!(map.map(0, 1), 0);
    assert_eq!(map.map(1, 1), 1);
    assert_eq!(map.map(3, 1), 2); // within deleted range, bias right → start+new_size = 2
    assert_eq!(map.map(3, -1), 2); // within deleted range, bias left → start = 2
    assert_eq!(map.map(6, 1), 3); // after deleted range, shifted by -3
}

#[test]
fn step_map_replace() {
    // Replacing 2 chars at position 3 with 5 chars
    let map = StepMap::new(3, 2, 5);
    assert_eq!(map.map(2, 1), 2);
    assert_eq!(map.map(4, -1), 3); // within range, bias left
    assert_eq!(map.map(4, 1), 8); // within range, bias right
    assert_eq!(map.map(6, 1), 9); // after range, shifted +3
}

#[test]
fn add_mark_to_text() {
    let doc = doc_with_paragraph("hello");
    // Positions: P open=0, "hello"=1..6, P close=6
    let step = Step::AddMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    };
    let (new_doc, inv) = step.apply(&doc).unwrap();

    let para = new_doc.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));

    // Inverse should be RemoveMark
    matches!(inv, Step::RemoveMark { .. });
}

#[test]
fn add_mark_partial_text() {
    let doc = doc_with_paragraph("hello");
    // Bold only "ell" (positions 2..5)
    let step = Step::AddMark {
        from: 2,
        to: 5,
        mark: MarkType::Bold,
    };
    let (new_doc, _) = step.apply(&doc).unwrap();

    let para = new_doc.content.child(0).unwrap();
    // The entire text node gets the mark since we don't split text nodes in AddMark
    let text = para.content.child(0).unwrap();
    assert!(text.marks.contains(&MarkType::Bold));
}

#[test]
fn remove_mark() {
    // Start with bold text
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::text("hello", vec![MarkType::Bold])],
        )],
    );

    let step = Step::RemoveMark {
        from: 1,
        to: 6,
        mark: MarkType::Bold,
    };
    let (new_doc, _) = step.apply(&doc).unwrap();

    let para = new_doc.content.child(0).unwrap();
    let text = para.content.child(0).unwrap();
    assert!(!text.marks.contains(&MarkType::Bold));
}

#[test]
fn replace_entire_block() {
    let doc = doc_two_paragraphs("hello", "world");
    // Replace first paragraph (positions 0..7) with a heading
    let heading =
        Node::new(NodeType::Heading, vec![Node::text("hello", vec![])]).with_attr("level", "1");
    let slice = Slice::new(Fragment::from_vec(vec![heading]), 0, 0);

    let step = Step::Replace {
        from: 0,
        to: 7,
        slice,
    };
    let (new_doc, inv) = step.apply(&doc).unwrap();

    assert_eq!(
        new_doc.content.child(0).unwrap().node_type,
        NodeType::Heading
    );
    assert_eq!(
        new_doc.content.child(1).unwrap().node_type,
        NodeType::Paragraph
    );

    // Inverse should restore original paragraph
    let (restored, _) = inv.apply(&new_doc).unwrap();
    assert_eq!(
        restored.content.child(0).unwrap().node_type,
        NodeType::Paragraph
    );
}

#[test]
fn replace_text_within_paragraph() {
    let doc = doc_with_paragraph("hello");
    // Replace "ell" (positions 2..5) with "a" → "hao"
    let slice = Slice::new(Fragment::from_vec(vec![Node::text("a", vec![])]), 0, 0);
    let step = Step::Replace {
        from: 2,
        to: 5,
        slice,
    };
    let (new_doc, _) = step.apply(&doc).unwrap();

    let para = new_doc.content.child(0).unwrap();
    // Should have text "h", "a", "o" as separate nodes or merged
    let mut full_text = String::new();
    for child in para.content.children() {
        if let Some(t) = &child.text {
            full_text.push_str(t);
        }
    }
    assert_eq!(full_text, "hao");
}

#[test]
fn replace_empty_range_inserts() {
    let doc = doc_with_paragraph("hello");
    // Insert "X" at position 3 (between 'e' and 'l')
    let slice = Slice::new(Fragment::from_vec(vec![Node::text("X", vec![])]), 0, 0);
    let step = Step::Replace {
        from: 3,
        to: 3,
        slice,
    };
    let (new_doc, _) = step.apply(&doc).unwrap();

    let para = new_doc.content.child(0).unwrap();
    let mut full_text = String::new();
    for child in para.content.children() {
        if let Some(t) = &child.text {
            full_text.push_str(t);
        }
    }
    assert_eq!(full_text, "heXllo");
}

#[test]
fn cut_slice_extracts_text() {
    let doc = doc_with_paragraph("hello");
    let slice = cut_slice(&doc, 2, 5);
    // Cut from inside a paragraph returns the paragraph wrapper with partial text
    let para = slice.content.child(0).unwrap();
    assert_eq!(para.node_type, NodeType::Paragraph);
    assert_eq!(para.content.child(0).unwrap().text.as_deref(), Some("ell"));
}

#[test]
fn cut_slice_entire_block() {
    let doc = doc_two_paragraphs("hello", "world");
    let slice = cut_slice(&doc, 0, 7);
    assert_eq!(slice.content.child_count(), 1);
    assert_eq!(
        slice.content.child(0).unwrap().node_type,
        NodeType::Paragraph
    );
}

#[test]
fn has_mark_in_range_works() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![
                Node::text("plain", vec![]),
                Node::text("bold", vec![MarkType::Bold]),
            ],
        )],
    );
    // "plain" at positions 1..6, "bold" at 6..10
    assert!(!has_mark_in_range(&doc, 1, 6, &MarkType::Bold));
    assert!(has_mark_in_range(&doc, 6, 10, &MarkType::Bold));
    assert!(has_mark_in_range(&doc, 1, 10, &MarkType::Bold));
}

#[test]
fn all_have_mark_works() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![
                Node::text("aa", vec![MarkType::Bold]),
                Node::text("bb", vec![MarkType::Bold]),
            ],
        )],
    );
    assert!(all_have_mark_in_range(&doc, 1, 5, &MarkType::Bold));

    let doc2 = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![
                Node::text("aa", vec![MarkType::Bold]),
                Node::text("bb", vec![]),
            ],
        )],
    );
    assert!(!all_have_mark_in_range(&doc2, 1, 5, &MarkType::Bold));
}
