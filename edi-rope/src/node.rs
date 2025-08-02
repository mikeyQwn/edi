//! Node of the rope's inner tree

use std::fmt::{Debug, Write};

use crate::iter::Lines;

/// A node in the rope binary tree.
pub(crate) enum Node {
    /// A leaf node contains an immutable string.
    /// Any operation that modifies the contained string should create new leaf nodes.
    Leaf {
        /// A part of the string that the rope represents
        value: Box<str>,
        /// Length of the `value` field in utf-8 characters
        char_len: usize,
        /// Total number of newlines in the string
        newlines: usize,
    },
    /// A value node contains a cumulative length of the left subtree leaf nodes' lengths.
    Value {
        /// Cumulative length of the left subtree leaf nodes' lengths a.k.a. `weight`
        left_len: usize,
        /// Cumulative length of the left subtree leaf nodes' newline counts
        left_newlines: usize,
        /// The left child of the node
        l: Option<Box<Node>>,
        /// The right child of the node
        r: Option<Box<Node>>,
    },
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_ascii_tree())
    }
}

impl Node {
    pub fn new_leaf(value: &str) -> Self {
        let char_len = value.chars().count();
        let newlines = value.chars().filter(|&c| c == '\n').count();
        let value = Box::from(value);
        Self::Leaf {
            value,
            char_len,
            newlines,
        }
    }

    /// Returns the weight of the node
    pub const fn weight(&self) -> usize {
        match self {
            Node::Leaf { char_len, .. } => *char_len,
            Node::Value { left_len: val, .. } => *val,
        }
    }

    /// Returns number of newlines of the node and all it's children combined
    pub const fn newlines(&self) -> usize {
        match self {
            Node::Leaf { newlines, .. } => *newlines,
            Node::Value { left_newlines, .. } => *left_newlines,
        }
    }

    /// Returns the weight of the node and all its children
    /// The full weight of the root node is the total number of characters in the rope.
    pub fn full_weight(&self) -> usize {
        match self {
            Node::Leaf { char_len, .. } => *char_len,
            Node::Value {
                left_len: val, r, ..
            } => {
                let r_weight = r.as_deref().map_or(0, Self::full_weight);
                val + r_weight
            }
        }
    }

    /// Returns number of newlines of the node and all it's children combined
    pub fn full_newlines(&self) -> usize {
        match self {
            Node::Leaf { newlines, .. } => *newlines,
            Node::Value {
                left_newlines, r, ..
            } => {
                let right_newlines = r.as_deref().map_or(0, Self::full_newlines);
                right_newlines + left_newlines
            }
        }
    }

    /// Returns a reference to the right child node, if any
    pub fn right(&self) -> Option<&Node> {
        match self {
            Node::Leaf { .. } => None,
            Node::Value { r, .. } => r.as_deref(),
        }
    }

    /// Returns a reference to the left child node, if any
    pub fn left(&self) -> Option<&Node> {
        match self {
            Node::Leaf { .. } => None,
            Node::Value { l, .. } => l.as_deref(),
        }
    }

    #[must_use]
    pub(crate) fn index_of_line(&self, line: usize) -> usize {
        Lines::new(self)
            .parse_contents(false)
            .nth(line)
            .map_or_else(|| self.full_weight(), |info| info.character_offset)
    }

    //// Returns an ASCII tree representation of the node and its children
    pub(crate) fn to_ascii_tree(&self) -> String {
        let mut buffer = String::new();
        self.build_ascii_tree(&mut buffer, "", "", false);
        buffer
    }

    fn build_ascii_tree(&self, buffer: &mut String, prefix: &str, child_prefix: &str, is_r: bool) {
        match self {
            Node::Leaf {
                value, char_len, ..
            } => {
                let content = if *char_len > 20 {
                    format!(
                        "{}... ({} chars)",
                        &value.chars().take(20).collect::<String>(),
                        char_len,
                    )
                } else {
                    format!("{:?} ({} chars)", value, value.len())
                };
                let _ = writeln!(
                    buffer,
                    "{prefix}Leaf ({}): {content}",
                    if is_r { "r" } else { "l" },
                );
            }
            Node::Value {
                left_len,
                left_newlines,
                l,
                r,
            } => {
                let _ = writeln!(
                    buffer,
                    "{prefix}Value ({}): left_len={left_len}, left_newlines={left_newlines}",
                    if is_r { "r" } else { "l" },
                );

                if let Some(left) = l {
                    if r.is_some() {
                        left.build_ascii_tree(
                            buffer,
                            &format!("{child_prefix}├── "),
                            &format!("{child_prefix}│   "),
                            false,
                        );
                    } else {
                        left.build_ascii_tree(
                            buffer,
                            &format!("{child_prefix}└── "),
                            &format!("{child_prefix}    "),
                            false,
                        );
                    }
                }

                if let Some(right) = r {
                    right.build_ascii_tree(
                        buffer,
                        &format!("{child_prefix}└── "),
                        &format!("{child_prefix}    "),
                        true,
                    );
                }
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::new_leaf("")
    }
}
