use super::*;

#[test]
fn text_node_size() {
    let t = Node::text("hello", vec![]);
    assert_eq!(t.node_size(), 5);
}

#[test]
fn paragraph_size() {
    let p = Node::new(NodeType::Paragraph, vec![Node::text("hi", vec![])]);
    assert_eq!(p.node_size(), 4);
}

#[test]
fn doc_size() {
    let doc = Node::new(
        NodeType::Doc,
        vec![Node::new(
            NodeType::Paragraph,
            vec![Node::text("hi", vec![])],
        )],
    );
    assert_eq!(doc.node_size(), 6);
}

#[test]
fn fragment_find_index() {
    let frag = Fragment::from_vec(vec![Node::text("ab", vec![]), Node::text("cd", vec![])]);
    assert_eq!(frag.find_index(0), Some((0, 0)));
    assert_eq!(frag.find_index(1), Some((0, 0)));
    assert_eq!(frag.find_index(2), Some((1, 2)));
    assert_eq!(frag.find_index(3), Some((1, 2)));
    assert_eq!(frag.find_index(4), Some((2, 4)));
    assert_eq!(frag.find_index(5), None);
}

#[test]
fn horizontal_rule_size() {
    let hr = Node::new(NodeType::HorizontalRule, vec![]);
    assert_eq!(hr.node_size(), 1);
}
