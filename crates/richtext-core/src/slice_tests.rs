use super::*;
use crate::node::{Fragment, Node};
use crate::schema::NodeType;

#[test]
fn empty_slice() {
    let s = Slice::empty();
    assert!(s.is_empty());
    assert_eq!(s.size(), 0);
    assert_eq!(s.open_start, 0);
    assert_eq!(s.open_end, 0);
}

#[test]
fn slice_with_content() {
    let frag = Fragment::from_vec(vec![Node::new(
        NodeType::Paragraph,
        vec![Node::text("hello", vec![])],
    )]);
    let s = Slice::new(frag, 0, 0);
    assert_eq!(s.size(), 7);
    assert!(!s.is_empty());
}

#[test]
fn open_slice() {
    let frag = Fragment::from_vec(vec![Node::text("ell", vec![])]);
    let s = Slice::new(frag, 1, 1);
    assert_eq!(s.open_start, 1);
    assert_eq!(s.open_end, 1);
    assert_eq!(s.size(), 3);
}
