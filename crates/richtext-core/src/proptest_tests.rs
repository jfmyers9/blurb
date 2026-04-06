use proptest::prelude::*;

use crate::markdown::{emit_markdown, parse_markdown};
use crate::node::Node;
use crate::position::resolve;
use crate::schema::NodeType;

fn arb_simple_text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,50}"
}

fn arb_paragraph_doc() -> impl Strategy<Value = Node> {
    prop::collection::vec(arb_simple_text(), 1..5).prop_map(|texts| {
        let paragraphs: Vec<Node> = texts
            .into_iter()
            .map(|t| Node::new(NodeType::Paragraph, vec![Node::text(&t, vec![])]))
            .collect();
        Node::new(NodeType::Doc, paragraphs)
    })
}

proptest! {
    #[test]
    fn position_resolve_in_bounds(doc in arb_paragraph_doc()) {
        let content_size = doc.content.size();
        for pos in 0..=content_size {
            let result = resolve(&doc, pos);
            prop_assert!(result.is_some(), "resolve({}) failed for doc with content_size={}", pos, content_size);
            let rp = result.unwrap();
            prop_assert!(rp.depth() <= 2, "unexpected depth {} at pos {}", rp.depth(), pos);
        }
        // Out of bounds should fail
        prop_assert!(resolve(&doc, content_size + 1).is_none());
    }

    #[test]
    fn position_resolve_depth_consistency(doc in arb_paragraph_doc()) {
        let content_size = doc.content.size();
        for pos in 0..=content_size {
            let rp = resolve(&doc, pos).unwrap();
            // Depth 0 should always be Doc
            prop_assert_eq!(rp.node_type_at(0), Some(NodeType::Doc));
        }
    }

    #[test]
    fn fragment_size_is_sum_of_children(doc in arb_paragraph_doc()) {
        let expected: usize = doc.content.children().iter().map(|c| c.node_size()).sum();
        prop_assert_eq!(doc.content.size(), expected);
    }

    #[test]
    fn markdown_roundtrip_simple_paragraphs(texts in prop::collection::vec(arb_simple_text(), 1..4)) {
        let input: String = texts.iter().map(|t| format!("{}\n", t.trim())).collect::<Vec<_>>().join("\n");
        let doc = parse_markdown(&input);
        let output = emit_markdown(&doc);
        let reparsed = parse_markdown(&output);
        let output2 = emit_markdown(&reparsed);
        // Idempotency: re-emitting should be stable
        prop_assert_eq!(&output, &output2, "markdown emission not idempotent");
    }
}
