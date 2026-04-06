use super::*;

#[test]
fn parse_simple_paragraph() {
    let doc = parse_markdown("Hello world");
    assert_eq!(doc.node_type, NodeType::Doc);
    assert_eq!(doc.child_count(), 1);
    let p = doc.content.child(0).unwrap();
    assert_eq!(p.node_type, NodeType::Paragraph);
    let t = p.content.child(0).unwrap();
    assert!(t.is_text());
    assert_eq!(t.text.as_deref(), Some("Hello world"));
}

#[test]
fn parse_bold_text() {
    let doc = parse_markdown("Hello **world**");
    let p = doc.content.child(0).unwrap();
    assert_eq!(p.content.child_count(), 2);
    let bold_text = p.content.child(1).unwrap();
    assert!(bold_text.marks.contains(&MarkType::Bold));
    assert_eq!(bold_text.text.as_deref(), Some("world"));
}

#[test]
fn parse_heading() {
    let doc = parse_markdown("## Title");
    let h = doc.content.child(0).unwrap();
    assert_eq!(h.node_type, NodeType::Heading);
    assert_eq!(h.attrs.get("level").map(|s| s.as_str()), Some("2"));
}

#[test]
fn parse_bullet_list() {
    let doc = parse_markdown("- a\n- b");
    let list = doc.content.child(0).unwrap();
    assert_eq!(list.node_type, NodeType::BulletList);
    assert_eq!(list.content.child_count(), 2);
}

#[test]
fn parse_blockquote() {
    let doc = parse_markdown("> quoted");
    let bq = doc.content.child(0).unwrap();
    assert_eq!(bq.node_type, NodeType::Blockquote);
}

#[test]
fn roundtrip_simple() {
    let input = "Hello world\n";
    let doc = parse_markdown(input);
    let output = emit_markdown(&doc);
    assert_eq!(output, input);
}

#[test]
fn roundtrip_heading() {
    let input = "## Title\n";
    let doc = parse_markdown(input);
    let output = emit_markdown(&doc);
    assert_eq!(output, input);
}

#[test]
fn roundtrip_bold() {
    let input = "Hello **world**\n";
    let doc = parse_markdown(input);
    let output = emit_markdown(&doc);
    assert_eq!(output, input);
}

#[test]
fn parse_horizontal_rule() {
    let doc = parse_markdown("---");
    let hr = doc.content.child(0).unwrap();
    assert_eq!(hr.node_type, NodeType::HorizontalRule);
}

#[test]
fn parse_code_block() {
    let doc = parse_markdown("```\nfn main() {}\n```");
    let cb = doc.content.child(0).unwrap();
    assert_eq!(cb.node_type, NodeType::CodeBlock);
}
