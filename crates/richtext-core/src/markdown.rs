use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::node::Node;
use crate::schema::{MarkType, NodeType};

/// Parse a Markdown string into a document Node.
pub fn parse_markdown(input: &str) -> Node {
    let options = Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(input, options);
    let mut builder = DocBuilder::new();

    for event in parser {
        builder.handle_event(event);
    }

    builder.finish()
}

/// Emit a document Node as a Markdown string.
pub fn emit_markdown(doc: &Node) -> String {
    let mut out = String::new();
    emit_node(doc, &mut out, 0);
    // Trim trailing newlines but keep one
    let trimmed = out.trim_end_matches('\n');
    let mut result = trimmed.to_string();
    result.push('\n');
    result
}

struct DocBuilder {
    /// Stack of open nodes being built. Each entry is (node_type, attrs, children, marks).
    stack: Vec<BuilderFrame>,
    /// Current active marks for inline content.
    active_marks: Vec<MarkType>,
}

struct BuilderFrame {
    node_type: NodeType,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

impl DocBuilder {
    fn new() -> Self {
        Self {
            stack: vec![BuilderFrame {
                node_type: NodeType::Doc,
                attrs: Vec::new(),
                children: Vec::new(),
            }],
            active_marks: Vec::new(),
        }
    }

    fn push_frame(&mut self, node_type: NodeType, attrs: Vec<(String, String)>) {
        self.stack.push(BuilderFrame {
            node_type,
            attrs,
            children: Vec::new(),
        });
    }

    fn pop_frame(&mut self) -> Option<Node> {
        let frame = self.stack.pop()?;
        let mut node = Node::new(frame.node_type, frame.children);
        for (k, v) in frame.attrs {
            node = node.with_attr(&k, &v);
        }
        Some(node)
    }

    fn add_child(&mut self, node: Node) {
        if let Some(frame) = self.stack.last_mut() {
            frame.children.push(node);
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.open_tag(tag),
            Event::End(tag) => self.close_tag(tag),
            Event::Text(text) => {
                let node = Node::text(&text, self.active_marks.clone());
                self.add_child(node);
            }
            Event::Code(code) => {
                let mut marks = self.active_marks.clone();
                marks.push(MarkType::Code);
                let node = Node::text(&code, marks);
                self.add_child(node);
            }
            Event::SoftBreak | Event::HardBreak => {
                let node = Node::text("\n", self.active_marks.clone());
                self.add_child(node);
            }
            Event::Rule => {
                self.add_child(Node::new(NodeType::HorizontalRule, vec![]));
            }
            _ => {}
        }
    }

    fn open_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.push_frame(NodeType::Paragraph, vec![]),
            Tag::Heading { level, .. } => {
                let lvl = heading_level_to_str(level);
                self.push_frame(NodeType::Heading, vec![("level".into(), lvl.into())]);
            }
            Tag::BlockQuote(_) => self.push_frame(NodeType::Blockquote, vec![]),
            Tag::CodeBlock(_) => self.push_frame(NodeType::CodeBlock, vec![]),
            Tag::List(first_item) => {
                if first_item.is_some() {
                    self.push_frame(NodeType::OrderedList, vec![]);
                } else {
                    self.push_frame(NodeType::BulletList, vec![]);
                }
            }
            Tag::Item => self.push_frame(NodeType::ListItem, vec![]),
            Tag::Emphasis => self.active_marks.push(MarkType::Italic),
            Tag::Strong => self.active_marks.push(MarkType::Bold),
            Tag::Strikethrough => self.active_marks.push(MarkType::Strike),
            Tag::Link {
                dest_url, title, ..
            } => {
                self.active_marks.push(MarkType::Link {
                    href: dest_url.to_string(),
                    title: if title.is_empty() {
                        None
                    } else {
                        Some(title.to_string())
                    },
                });
            }
            _ => {}
        }
    }

    fn close_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
            | TagEnd::List(_)
            | TagEnd::Item => {
                if let Some(node) = self.pop_frame() {
                    // If a ListItem has inline children but no block wrapper, wrap in Paragraph
                    let node = if node.node_type == NodeType::ListItem {
                        wrap_list_item_inline(node)
                    } else {
                        node
                    };
                    self.add_child(node);
                }
            }
            TagEnd::Emphasis => {
                self.active_marks.retain(|m| !matches!(m, MarkType::Italic));
            }
            TagEnd::Strong => {
                self.active_marks.retain(|m| !matches!(m, MarkType::Bold));
            }
            TagEnd::Strikethrough => {
                self.active_marks.retain(|m| !matches!(m, MarkType::Strike));
            }
            TagEnd::Link => {
                self.active_marks
                    .retain(|m| !matches!(m, MarkType::Link { .. }));
            }
            _ => {}
        }
    }

    fn finish(mut self) -> Node {
        // Close any remaining frames
        while self.stack.len() > 1 {
            if let Some(node) = self.pop_frame() {
                self.add_child(node);
            }
        }
        self.pop_frame()
            .unwrap_or_else(|| Node::new(NodeType::Doc, vec![]))
    }
}

/// Wrap inline children of a ListItem in a Paragraph if they aren't already block nodes.
fn wrap_list_item_inline(node: Node) -> Node {
    let has_only_inline = node
        .content
        .children()
        .iter()
        .all(|c| c.node_type.is_inline() || c.node_type == NodeType::Text);

    if has_only_inline && !node.content.is_empty() {
        let children: Vec<Node> = node.content.children().to_vec();
        Node::new(
            NodeType::ListItem,
            vec![Node::new(NodeType::Paragraph, children)],
        )
    } else {
        node
    }
}

fn heading_level_to_str(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "1",
        HeadingLevel::H2 => "2",
        HeadingLevel::H3 => "3",
        HeadingLevel::H4 => "4",
        HeadingLevel::H5 => "5",
        HeadingLevel::H6 => "6",
    }
}

// --- Markdown emission ---

fn emit_node(node: &Node, out: &mut String, indent: usize) {
    match node.node_type {
        NodeType::Doc => {
            emit_children_blocks(node, out, indent);
        }
        NodeType::Paragraph => {
            emit_inline_content(node, out);
            out.push('\n');
        }
        NodeType::Heading => {
            let level: usize = node
                .attrs
                .get("level")
                .and_then(|l| l.parse().ok())
                .unwrap_or(1);
            for _ in 0..level {
                out.push('#');
            }
            out.push(' ');
            emit_inline_content(node, out);
            out.push('\n');
        }
        NodeType::Blockquote => {
            let mut inner = String::new();
            emit_children_blocks(node, &mut inner, 0);
            for line in inner.lines() {
                out.push_str("> ");
                out.push_str(line);
                out.push('\n');
            }
        }
        NodeType::CodeBlock => {
            out.push_str("```\n");
            emit_inline_content(node, out);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
        }
        NodeType::BulletList => {
            for child in node.content.children() {
                out.push_str("- ");
                emit_list_item_content(child, out, indent + 2);
            }
        }
        NodeType::OrderedList => {
            for (i, child) in node.content.children().iter().enumerate() {
                let prefix = format!("{}. ", i + 1);
                out.push_str(&prefix);
                emit_list_item_content(child, out, indent + prefix.len());
            }
        }
        NodeType::ListItem => {
            emit_children_blocks(node, out, indent);
        }
        NodeType::HorizontalRule => {
            out.push_str("---\n");
        }
        NodeType::Text => {
            // Handled by emit_inline_content
        }
    }
}

fn emit_children_blocks(node: &Node, out: &mut String, indent: usize) {
    let children = node.content.children();
    for (i, child) in children.iter().enumerate() {
        emit_node(child, out, indent);
        // Add blank line between block nodes (except last)
        if i + 1 < children.len() && !child.node_type.is_inline() {
            out.push('\n');
        }
    }
}

fn emit_list_item_content(node: &Node, out: &mut String, indent: usize) {
    let children = node.content.children();
    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            // Indent continuation blocks
            for _ in 0..indent {
                out.push(' ');
            }
        }
        emit_node(child, out, indent);
        if i + 1 < children.len() {
            out.push('\n');
        }
    }
}

fn emit_inline_content(node: &Node, out: &mut String) {
    for child in node.content.children() {
        if child.is_text() {
            let text = child.text.as_deref().unwrap_or("");
            let mut wrapped = String::from(text);

            // Apply marks in reverse order for proper nesting
            for mark in child.marks.iter().rev() {
                wrapped = apply_mark_wrapper(mark, &wrapped);
            }
            out.push_str(&wrapped);
        }
    }
}

fn apply_mark_wrapper(mark: &MarkType, text: &str) -> String {
    match mark {
        MarkType::Bold => format!("**{text}**"),
        MarkType::Italic => format!("*{text}*"),
        MarkType::Strike => format!("~~{text}~~"),
        MarkType::Code => format!("`{text}`"),
        MarkType::Link { href, title } => {
            if let Some(t) = title {
                format!("[{text}]({href} \"{t}\")")
            } else {
                format!("[{text}]({href})")
            }
        }
    }
}

#[cfg(test)]
#[path = "markdown_tests.rs"]
mod tests;
