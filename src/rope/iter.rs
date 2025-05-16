//! `Rope` iterators

use std::{
    iter::{Enumerate, Peekable, Skip, Take},
    ops::Range,
};

use crate::debug;

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

#[derive(Debug)]
struct CharsNode<'a> {
    tree_node: &'a Node,
    offset_from_start: usize,
    newlines_from_start: usize,
}

impl<'a> CharsNode<'a> {
    pub fn new(tree_node: &'a Node, offset_from_start: usize, newlines_from_start: usize) -> Self {
        Self {
            tree_node,
            offset_from_start,
            newlines_from_start,
        }
    }
}

/// An iterator over string characters that the `Node` represents
///
/// This iterator traverses `Rope`'s `Node` tree in-order and returns characters met, effectively
/// "streaming" the `Rope` contents
#[derive(Debug)]
pub struct Chars<'a> {
    stack: Vec<CharsNode<'a>>,
    current_node_offset_b: usize,
    global_character_offset: usize,
}

impl<'a> Chars<'a> {
    pub(super) fn new(node: &'a Node) -> Self {
        let mut it = Self {
            stack: vec![],
            current_node_offset_b: 0,
            global_character_offset: 0,
        };
        it.push_left(Some(CharsNode::new(node, 0, 0)));
        it
    }

    fn push_left(&mut self, mut it: Option<CharsNode<'a>>) {
        while let Some(value) = it {
            let node_weight = value.tree_node.weight();
            let newlines_from_start = value.tree_node.newlines();
            it = value
                .tree_node
                .left()
                .map(|node| CharsNode::new(node, node_weight, newlines_from_start));
            self.stack.push(value);
        }
    }
}

impl Iterator for Chars<'_> {
    type Item = char;

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n == 0 {
            return self.next();
        }

        let target = self.global_character_offset + n;
        let mut skipped_node = false;
        while let Some(node) = self.stack.last() {
            if node.offset_from_start + node.tree_node.weight() >= target {
                break;
            }

            self.stack.pop();
            skipped_node = true;
        }

        if skipped_node {
            self.global_character_offset = self.stack.last()?.tree_node.weight();
        }

        while target - self.global_character_offset > 0 {
            let _ = self.next()?;
        }

        self.next()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.last()?;

        match node.tree_node {
            Node::Leaf(leaf) => {
                let s = &leaf[self.current_node_offset_b..];
                let Some(char) = s.chars().next() else {
                    self.current_node_offset_b = 0;
                    self.stack.pop();
                    return self.next();
                };
                self.current_node_offset_b += char.len_utf8();
                self.global_character_offset += 1;
                Some(char)
            }
            Node::Value {
                left_len,
                left_newlines,
                r,
                ..
            } => {
                self.current_node_offset_b = 0;
                let offs = node.offset_from_start + left_len;
                let newlines = node.newlines_from_start + left_newlines;
                self.stack.pop();
                self.push_left(
                    r.as_ref()
                        .map(|tree_node| CharsNode::new(tree_node, offs, newlines)),
                );
                self.next()
            }
        }
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
    parent: &'a Node,
    parse_contents: bool,
    newlines_seen: usize,
    chars_skipped: usize,
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

        Self::from_raw(iter, n)
    }

    const fn from_raw(iter: Peekable<Enumerate<Chars<'a>>>, parent: &'a Node) -> Self {
        Self {
            iter,
            parent,
            parse_contents: true,
            newlines_seen: 0,
            chars_skipped: 0,
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
        let character_offset = self.chars_skipped + character_offset;
        let line_number = self.newlines_seen;
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

        self.newlines_seen += 1;

        Some(LineInfo {
            line_number,
            character_offset,
            length,
            contents,
        })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n == 0 {
            return self.next();
        }
        let parse_setting = self.parse_contents;

        for _ in 0..n {
            let _ = self.next();
        }

        self.parse_contents = parse_setting;
        self.next()
    }
}

#[cfg(test)]
mod tests {

    use crate::rope::{Chars, Rope};

    #[test]
    fn chars_empty() {
        let r = Rope::from("");
        let mut it = Chars::new(&r.root);
        assert_eq!(it.next(), None);
        assert_eq!(it.nth(0), None);
    }

    #[test]
    fn chars_forward() {
        let r = Rope::from("hello world");
        let mut it = Chars::new(&r.root);
        assert_eq!(it.next(), Some('h'));
        assert_eq!(it.next(), Some('e'));
        assert_eq!(it.next(), Some('l'));
        assert_eq!(it.next(), Some('l'));
        assert_eq!(it.next(), Some('o'));
        assert_eq!(it.next(), Some(' '));
        assert_eq!(it.next(), Some('w'));
        assert_eq!(it.next(), Some('o'));
        assert_eq!(it.next(), Some('r'));
        assert_eq!(it.next(), Some('l'));
        assert_eq!(it.next(), Some('d'));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn chars_nth() {
        let r = Rope::from("hello world");
        let mut it = Chars::new(&r.root);
        assert_eq!(it.nth(4), Some('o'));
        assert_eq!(it.next(), Some(' '));
        assert_eq!(it.next(), Some('w'));
        assert_eq!(it.nth(3), Some('d'));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn chars_skip_nth() {
        let r = Rope::from("hello");
        let mut it = Chars::new(&r.root);
        it.by_ref().skip(1).next();
        assert_eq!(it.next(), Some('l'));
        assert_eq!(it.next(), Some('l'));
        assert_eq!(it.nth(0), Some('o'));
        assert_eq!(it.nth(0), None);
    }

    #[test]
    fn chars_unicode() {
        let r = Rope::from("こんにちは世界");
        let mut it = Chars::new(&r.root);
        assert_eq!(it.next(), Some('こ'));
        assert_eq!(it.next(), Some('ん'));
        assert_eq!(it.nth(1), Some('ち'));
        assert_eq!(it.next(), Some('は'));
        assert_eq!(it.next(), Some('世'));
        assert_eq!(it.next(), Some('界'));
    }

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
            "\n\nline 3\n\nline 5\n",
        ];
        for input in inputs {
            let r = Rope::from(input);

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

    #[test]
    fn lines_nth_and_next() {
        let input = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6";
        let rope = Rope::from(input);
        let mut lines = rope.lines();

        assert_eq!(lines.nth(2).unwrap().contents, "line 3");
        assert_eq!(lines.next().unwrap().contents, "line 4"); // Should continue after nth

        assert_eq!(lines.nth(0).unwrap().contents, "line 5");
        assert_eq!(lines.next().unwrap().contents, "line 6");

        assert!(lines.nth(10).is_none());
        assert!(lines.next().is_none()); // Should stay at end

        let mut lines = rope.lines();
        assert_eq!(lines.next().unwrap().contents, "line 1");
        assert_eq!(lines.nth(1).unwrap().contents, "line 3"); // Skip line 2
        assert_eq!(lines.next().unwrap().contents, "line 4");
        assert_eq!(lines.nth(0).unwrap().contents, "line 5"); // Equivalent to next()
        assert_eq!(lines.next().unwrap().contents, "line 6");

        let input = "\n\nline 3\n\nline 5\n";
        let rope = Rope::from(input);
        let mut lines = rope.lines();

        assert_eq!(lines.nth(2).unwrap().contents, "line 3");
        assert_eq!(lines.next().unwrap().contents, ""); // Empty line 4
        assert_eq!(lines.nth(0).unwrap().contents, "line 5");
        assert!(lines.next().is_none());
    }

    #[test]
    fn lines_nth_and_next_without_contents() {
        let input = "line 1\nline 2\nline 3";
        let rope = Rope::from(input);
        let mut lines = rope.lines();
        let lines = lines.parse_contents(false);

        assert_eq!(lines.nth(1).unwrap().line_number, 1);
        assert_eq!(lines.next().unwrap().line_number, 2);
        assert!(lines.next().is_none());
    }
}
