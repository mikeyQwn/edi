use std::{
    iter::{FlatMap, Skip, Take},
    ops::Range,
};

use super::Node;

#[derive(Default)]
pub struct InorderIter<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> InorderIter<'a> {
    pub(super) fn new(r: &'a Node) -> Self {
        let it: Option<&Node> = Some(r);
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

pub struct Chars<'a>(FlatMap<InorderIter<'a>, std::str::Chars<'a>, fn(&str) -> std::str::Chars>);

impl<'a> Chars<'a> {
    pub(super) fn new(r: &'a Node) -> Self {
        let out: FlatMap<InorderIter, std::str::Chars, fn(&str) -> std::str::Chars> =
            InorderIter::new(r).flat_map(str::chars);
        Self(out)
    }
}

impl Iterator for Chars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct Substring<'a>(Take<Skip<Chars<'a>>>);

impl<'a> Substring<'a> {
    pub fn new(it: Chars<'a>, range: Range<usize>) -> Self {
        Self(it.skip(range.start).take(range.len()))
    }
}

impl Iterator for Substring<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
