use super::*;
use crate::node::Node;
use crate::schema::NodeType;

fn sample_doc() -> Node {
    Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::text("hi", vec![])],
        )],
    )
}

#[test]
fn resolve_inside_paragraph() {
    let doc = sample_doc();
    let rp = resolve(&doc, 1).unwrap();
    assert_eq!(rp.depth(), 1);
    assert_eq!(rp.node_type_at(0), Some(NodeType::Doc));
    assert_eq!(rp.node_type_at(1), Some(NodeType::Paragraph));
    assert_eq!(rp.index_at(0), Some(0));
}

#[test]
fn resolve_inside_text() {
    let doc = sample_doc();
    let rp = resolve(&doc, 2).unwrap();
    assert_eq!(rp.depth(), 1);
    assert_eq!(rp.node_type_at(1), Some(NodeType::Paragraph));
    assert_eq!(rp.index_at(1), Some(0));
}

#[test]
fn resolve_end_of_doc() {
    let doc = sample_doc();
    let rp = resolve(&doc, 4).unwrap();
    assert_eq!(rp.depth(), 0);
    assert_eq!(rp.node_type_at(0), Some(NodeType::Doc));
}

#[test]
fn resolve_out_of_bounds() {
    let doc = sample_doc();
    assert!(resolve(&doc, 5).is_none());
}

#[test]
fn resolve_multi_paragraph() {
    let doc = Node::new(
        NodeType::Doc,
        vec![
            Node::new(NodeType::Paragraph, vec![Node::text("ab", vec![])]),
            Node::new(NodeType::Paragraph, vec![Node::text("cd", vec![])]),
        ],
    );
    assert_eq!(doc.content.size(), 8);

    let rp = resolve(&doc, 5).unwrap();
    assert_eq!(rp.depth(), 1);
    assert_eq!(rp.node_type_at(1), Some(NodeType::Paragraph));
    assert_eq!(rp.index_at(0), Some(1));
}
