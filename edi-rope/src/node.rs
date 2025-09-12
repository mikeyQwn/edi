//! Node of the rope's inner tree

use crate::{leaf::Leaf, value::Value};

/// A node in the rope binary tree.
pub(crate) enum Node {
    /// A leaf node contains a string that might be mutated, but the whole subtree
    /// is supposed to be updated then
    Leaf(Leaf),
    /// A SOA containing all the children and their information
    Value(Value),
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_ascii_tree())
    }
}

impl Node {
    pub fn empty_leaf() -> Self {
        Self::Leaf(Leaf::empty())
    }

    pub fn new_leaf(s: &str) -> Self {
        Self::Leaf(Leaf::new(s))
    }

    pub fn empty_value() -> Self {
        Self::Value(Value::empty())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Leaf(leaf) => Some(leaf.as_str()),
            Self::Value { .. } => None,
        }
    }

    /// Returns the weight of the node
    pub const fn weight(&self) -> usize {
        match self {
            Self::Leaf(leaf) => leaf.weight(),
            Self::Value(value) => value.weight(),
        }
    }

    /// Returns number of newlines of the node
    pub const fn newlines(&self) -> usize {
        match self {
            Self::Leaf(leaf) => leaf.newlines(),
            Self::Value(value) => value.newlines(),
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            Self::Leaf(_) => 0,
            Self::Value(value) => {
                1 + value
                    .children()
                    .iter()
                    .flatten()
                    .map(|node| node.depth())
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    /// Returns a an example node tree
    pub fn example() -> Self {
        todo!("implement me")
    }

    // #[must_use]
    // pub(crate) fn index_of_line(&self, line: usize) -> usize {
    //     Lines::new(self)
    //         .parse_contents(false)
    //         .nth(line)
    //         .map_or_else(|| self.full_weight(), |info| info.character_offset)
    // }

    /// Returns an ASCII tree representation of the node and its children
    pub fn to_ascii_tree(&self) -> String {
        let mut buffer = String::new();
        self.build_ascii_tree(&mut buffer, "", "", 0);
        buffer
    }

    fn build_ascii_tree(&self, buffer: &mut String, prefix: &str, child_prefix: &str, num: usize) {
        use std::fmt::Write;

        const TRIM_LENGTH: usize = 20;
        match self {
            Self::Leaf(leaf) => {
                buffer.push_str(prefix);
                buffer.push_str("Leaf (");
                let _ = write!(buffer, "{}", num);
                buffer.push_str("): ");
                buffer.push_str(crate::string::cut_to_chars(leaf.as_str(), TRIM_LENGTH));
                if leaf.info().chars > TRIM_LENGTH {
                    buffer.push_str("...");
                }
                buffer.push_str(" (");
                let _ = write!(buffer, "{}", leaf.info().chars);
                buffer.push_str("chars )");
            }
            Self::Value(value) => {
                let _ = writeln!(
                    buffer,
                    "{prefix}Value ({}): weight={weight}",
                    num,
                    weight = value.weight()
                );
                for (i, child) in value.children().iter().enumerate() {
                    child
                        .as_ref()
                        .expect("children up to len must be initialized")
                        .build_ascii_tree(
                            buffer,
                            &format!("{child_prefix}├── "),
                            &format!("{child_prefix}│   "),
                            i,
                        );
                }
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::empty_leaf()
    }
}

#[cfg(test)]
mod tests {}
