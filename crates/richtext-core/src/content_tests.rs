use super::*;
use crate::node::Node;
use crate::schema::{MarkType, NodeType};

#[test]
fn valid_doc() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::text("hello", vec![])],
        )],
    );
    assert!(validate_content(&doc).is_ok());
}

#[test]
fn text_in_doc_is_invalid() {
    let doc = Node::new(NodeType::Doc, vec![Node::text("raw text", vec![])]);
    assert!(validate_content(&doc).is_err());
}

#[test]
fn block_in_paragraph_is_invalid() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::new(NodeType::Paragraph, vec![])],
        )],
    );
    assert!(validate_content(&doc).is_err());
}

#[test]
fn valid_list() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::BulletList,
            vec![Node::new(
                NodeType::ListItem,
                vec![Node::new(
                    NodeType::Paragraph,
                    vec![Node::text("item", vec![])],
                )],
            )],
        )],
    );
    assert!(validate_content(&doc).is_ok());
}

#[test]
fn valid_marks() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![
                Node::text("hello ", vec![]),
                Node::text("world", vec![MarkType::Bold]),
            ],
        )],
    );
    assert!(validate_content(&doc).is_ok());
}

#[test]
fn hr_with_children_is_invalid() {
    let mut hr = Node::new(NodeType::HorizontalRule, vec![]);
    hr.content.push(Node::text("oops", vec![]));
    assert!(validate_content(&hr).is_err());
}
