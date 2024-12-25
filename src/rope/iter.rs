use std::{
    iter::{FlatMap, TakeWhile},
    str::Chars,
};

use super::{Node, Rope};

#[derive(Default)]
pub struct InorderIter<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> InorderIter<'a> {
    pub fn new(r: &'a Rope) -> Self {
        let it: Option<&Node> = Some(&r.root);
        let mut out = Self::default();
        out.push_left(it);

        out
    }

    fn push_left(&mut self, mut it: Option<&'a Node>) {
        while let Some(value) = it {
            self.stack.push(value);
            it = value.left();
        }
    }
}

impl<'a> Iterator for InorderIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let string = match self.stack.pop() {
            Some(Node::Leaf(string)) => string,
            Some(v) => {
                self.push_left(v.right());
                return self.next();
            }
            None => return None,
        };

        if self.stack.is_empty() {
            return Some(string);
        }

        // Take the right child of the current node and push all its left children onto the stack
        let Some(Node::Value { r: Some(r), .. }) = self.stack.pop() else {
            return Some(string);
        };

        self.push_left(Some(r));

        Some(string)
    }
}

// TODO: Rename to chars
pub struct CharsIter<'a>(FlatMap<InorderIter<'a>, Chars<'a>, fn(&str) -> Chars>);

impl<'a> CharsIter<'a> {
    pub fn new(r: &'a Rope) -> Self {
        let out: FlatMap<InorderIter, Chars, fn(&str) -> Chars> =
            InorderIter::new(r).flat_map(str::chars);
        Self(out)
    }
}

impl Iterator for CharsIter<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
