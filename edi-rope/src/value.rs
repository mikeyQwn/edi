use crate::{info::TextInfo, node::Node};

const CHILD_COUNT: usize = 4;

pub struct Value {
    /// INVARIANT: Always less or equal to `CHILD_COUNT`.
    /// If the node is not root, len must be more or equal to `CHILD_COUNT / 2`
    len: usize,
    children: [Option<Box<Node>>; CHILD_COUNT],
    infos: [TextInfo; CHILD_COUNT],
}

impl Value {
    pub fn empty() -> Self {
        Self {
            len: 0,
            children: [const { None }; CHILD_COUNT],
            infos: [TextInfo::empty(); CHILD_COUNT],
        }
    }

    pub const fn weight(&self) -> usize {
        let mut sum = 0;
        let mut i = 0;
        while i < self.len {
            sum += self.infos[i].chars;
            i += 1;
        }
        sum
    }

    pub const fn newlines(&self) -> usize {
        let mut sum = 0;
        let mut i = 0;
        while i < self.len {
            sum += self.infos[i].newlines;
            i += 1;
        }
        sum
    }

    pub fn children(&self) -> &[Option<Box<Node>>] {
        &self.children[..self.len]
    }
}
