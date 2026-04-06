use serde::{Deserialize, Serialize};

/// Block and inline node types following ProseMirror's schema model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Doc,
    Paragraph,
    Heading,
    BulletList,
    OrderedList,
    ListItem,
    Blockquote,
    CodeBlock,
    HorizontalRule,
    Text,
}

/// Mark types that can be applied to inline text ranges.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarkType {
    Bold,
    Italic,
    Strike,
    Link { href: String, title: Option<String> },
    Code,
}

/// Content expression describing what children a node type allows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentExpr {
    /// No children allowed (leaf nodes).
    Empty,
    /// Allows zero or more block nodes.
    Block,
    /// Allows zero or more inline nodes (Text with marks).
    Inline,
}

impl NodeType {
    /// Returns the content expression for this node type.
    pub fn content_expr(&self) -> ContentExpr {
        match self {
            NodeType::Doc => ContentExpr::Block,
            NodeType::Paragraph => ContentExpr::Inline,
            NodeType::Heading => ContentExpr::Inline,
            NodeType::BulletList => ContentExpr::Block, // contains ListItems
            NodeType::OrderedList => ContentExpr::Block, // contains ListItems
            NodeType::ListItem => ContentExpr::Block,   // contains block content
            NodeType::Blockquote => ContentExpr::Block,
            NodeType::CodeBlock => ContentExpr::Inline,
            NodeType::HorizontalRule => ContentExpr::Empty,
            NodeType::Text => ContentExpr::Empty,
        }
    }

    /// Whether this is an inline node type.
    pub fn is_inline(&self) -> bool {
        matches!(self, NodeType::Text)
    }

    /// Whether this is a block node type.
    pub fn is_block(&self) -> bool {
        !self.is_inline() && !matches!(self, NodeType::Doc)
    }

    /// Whether this is a leaf node (no children).
    pub fn is_leaf(&self) -> bool {
        self.content_expr() == ContentExpr::Empty
    }
}
