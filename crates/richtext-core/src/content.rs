use crate::node::Node;
use crate::schema::{ContentExpr, NodeType};

/// Validate that a node's children conform to its content expression.
pub fn validate_content(node: &Node) -> Result<(), ContentError> {
    let expr = node.node_type.content_expr();

    match expr {
        ContentExpr::Empty => {
            if !node.content.is_empty() && !node.is_text() {
                return Err(ContentError::NoChildrenAllowed {
                    node_type: node.node_type,
                });
            }
        }
        ContentExpr::Block => {
            for (i, child) in node.content.children().iter().enumerate() {
                if child.is_text() {
                    return Err(ContentError::InvalidChild {
                        parent: node.node_type,
                        child: child.node_type,
                        index: i,
                        reason: "block node cannot contain text directly",
                    });
                }
                if child.node_type.is_inline() {
                    return Err(ContentError::InvalidChild {
                        parent: node.node_type,
                        child: child.node_type,
                        index: i,
                        reason: "block node cannot contain inline content",
                    });
                }
            }
        }
        ContentExpr::Inline => {
            for (i, child) in node.content.children().iter().enumerate() {
                if !child.node_type.is_inline() && child.node_type != NodeType::Text {
                    return Err(ContentError::InvalidChild {
                        parent: node.node_type,
                        child: child.node_type,
                        index: i,
                        reason: "inline context cannot contain block nodes",
                    });
                }
            }
        }
    }

    // Recursively validate children
    for child in node.content.children() {
        validate_content(child)?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum ContentError {
    NoChildrenAllowed {
        node_type: NodeType,
    },
    InvalidChild {
        parent: NodeType,
        child: NodeType,
        index: usize,
        reason: &'static str,
    },
}

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentError::NoChildrenAllowed { node_type } => {
                write!(f, "{node_type:?} does not allow children")
            }
            ContentError::InvalidChild {
                parent,
                child,
                index,
                reason,
            } => {
                write!(
                    f,
                    "invalid child {child:?} at index {index} in {parent:?}: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for ContentError {}

#[cfg(test)]
#[path = "content_tests.rs"]
mod tests;
