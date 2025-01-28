//! `Rope` iterators

use std::{
    iter::{Enumerate, FlatMap, Peekable, Skip, Take},
    ops::Range,
};

use super::Node;

/// An iterator that does inorder `Rope` traversal starting from given `Node` and returns `Node`s met
#[derive(Default, Debug)]
pub(super) struct InorderIter<'a> {
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

/// An iterator over string characters that the `Node` represents
///
/// This iterator traverses `Rope`'s `Node` tree in-order and returns characters met, effectively
/// "streaming" the `Rope` contents
#[derive(Debug)]
pub struct Chars<'a>(FlatMap<InorderIter<'a>, std::str::Chars<'a>, fn(&str) -> std::str::Chars>);

impl<'a> Chars<'a> {
    pub(super) fn new(r: &'a Node) -> Self {
        let out: FlatMap<InorderIter, std::str::Chars, fn(&str) -> std::str::Chars> =
            InorderIter::new(r).flat_map(str::chars);
        Self(out)
    }

    /// Converts interator over chars to iterator over lines
    pub fn lines(self) -> Lines<'a> {
        Lines::from_raw(self.enumerate().peekable())
    }
}

impl Iterator for Chars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A substring iterator that is returned after calling `substr` method of `Rope`
#[derive(Debug)]
pub struct Substring<'a>(Take<Skip<Chars<'a>>>);

impl<'a> Substring<'a> {
    /// Initializes `Substring` using the `Chars` and a substring range
    #[must_use]
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

/// An iterator over lines of the `Rope`
///
/// Can be modified to not parse the contents of the string into `contents` field of `LineInfo`
/// returned
#[derive(Debug)]
pub struct Lines<'a> {
    iter: Peekable<Enumerate<Chars<'a>>>,
    parse_contents: bool,
    seen_count: usize,
}

/// Represents information about a string line
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineInfo {
    /// Zero-indexed line number
    pub line_number: usize,
    /// Offset from the start of the string
    pub character_offset: usize,
    /// A total number of characters in a string line, NOT number of bytes that the line takes
    pub length: usize,
    /// A string representation of the line. It will be empty if `parse_contents` is `false` OR if
    /// line is actually empty
    pub contents: String,
}

impl<'a> Lines<'a> {
    #[must_use]
    pub(super) fn new(n: &'a Node) -> Self {
        let iter = Chars::new(n).enumerate().peekable();

        Self::from_raw(iter)
    }

    const fn from_raw(iter: Peekable<Enumerate<Chars<'a>>>) -> Self {
        Self {
            iter,
            parse_contents: true,
            seen_count: 0,
        }
    }

    /// Modifies the iterator to not include the string representing the line in it's output
    pub fn parse_contents(&mut self, parse_contents: bool) -> &mut Self {
        self.parse_contents = parse_contents;
        self
    }
}

impl Iterator for Lines<'_> {
    type Item = LineInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let &(character_offset, _) = self.iter.peek()?;
        let line_number = self.seen_count;
        let (contents, length) = if self.parse_contents {
            self.iter
                .by_ref()
                .take_while(|&(_, char)| char != '\n')
                .map(|(_, char)| char)
                .fold((String::new(), 0), |(mut string, len), curr| {
                    string.push(curr);
                    (string, len + 1)
                })
        } else {
            let len = self
                .iter
                .by_ref()
                .take_while(|&(_, char)| char != '\n')
                .count();
            (String::new(), len)
        };

        self.seen_count += 1;

        Some(LineInfo {
            line_number,
            character_offset,
            length,
            contents,
        })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let parse_setting = self.parse_contents;
        self.parse_contents = false;

        for _ in 0..n {
            let _ = self.next()?;
        }

        self.parse_contents = parse_setting;
        self.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::rope::Rope;

    #[test]
    fn lines() {
        let inputs = [
            "",
            "a\nb\nbc\nd\n",
            "\n\n\n",
            "a\n",
            "\na",
            "\na\n",
            "\naba\n",
            "\n",
        ];
        for input in inputs {
            println!("{:?}", input);
            let r = Rope::from(String::from(input));

            assert_eq!(
                r.lines().map(|i| i.contents).collect::<Vec<_>>(),
                input.lines().collect::<Vec<_>>()
            );

            assert_eq!(
                r.lines()
                    .parse_contents(false)
                    .map(|i| i.contents)
                    .collect::<Vec<_>>(),
                input.lines().map(|_| String::new()).collect::<Vec<_>>()
            );
        }
    }
}
