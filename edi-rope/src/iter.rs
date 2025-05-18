//! `Rope` iterators

use std::{
    iter::{Skip, Take},
    ops::Range,
};

use super::Node;

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
    global_line_offset: usize,
}

impl<'a> Chars<'a> {
    pub(super) fn new(node: &'a Node) -> Self {
        let mut it = Self {
            stack: vec![],
            current_node_offset_b: 0,
            global_character_offset: 0,
            global_line_offset: 0,
        };
        it.push_left(Some(CharsNode::new(node, 0, 0)));
        it
    }

    fn push_left(&mut self, mut it: Option<CharsNode<'a>>) {
        while let Some(value) = it {
            let offset_from_start = value.offset_from_start;
            let newlines_from_start = value.newlines_from_start;
            it = value
                .tree_node
                .left()
                .map(|node| CharsNode::new(node, offset_from_start, newlines_from_start));
            self.stack.push(value);
        }
    }

    fn skip_lines(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        let target = self.global_line_offset + n;

        let mut skipped_node = false;
        while let Some(node) = self.stack.last() {
            if node.newlines_from_start + node.tree_node.full_newlines() >= target {
                break;
            }

            let value = self.stack.pop().and_then(|v| {
                Some(CharsNode::new(
                    v.tree_node.right()?,
                    v.offset_from_start + v.tree_node.weight(),
                    v.newlines_from_start + v.tree_node.newlines(),
                ))
            });
            self.push_left(value);
            skipped_node = true;
        }

        if skipped_node {
            let Some(node) = self.stack.last() else {
                return;
            };
            self.global_character_offset = node.offset_from_start;
            self.global_line_offset = node.newlines_from_start;
            self.current_node_offset_b = 0;
        }

        while target - self.global_line_offset > 0 {
            if self.next().is_none() {
                return;
            };
        }
    }

    #[must_use]
    const fn characters_consumed(&self) -> usize {
        self.global_character_offset
    }

    #[must_use]
    const fn newlines_consumed(&self) -> usize {
        self.global_line_offset
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
            if node.offset_from_start + node.tree_node.full_weight() >= target {
                break;
            }

            let value = self.stack.pop().and_then(|v| {
                Some(CharsNode::new(
                    v.tree_node.right()?,
                    v.offset_from_start + v.tree_node.weight(),
                    v.newlines_from_start + v.tree_node.newlines(),
                ))
            });
            self.push_left(value);
            skipped_node = true;
        }

        if skipped_node {
            let node = self.stack.last()?;
            self.global_character_offset = node.offset_from_start;
            self.global_line_offset = node.newlines_from_start;
            self.current_node_offset_b = 0;
        }

        while target - self.global_character_offset > 0 {
            self.next();
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
                if char == '\n' {
                    self.global_line_offset += 1;
                }
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
                self.global_character_offset = offs;
                let newlines = node.newlines_from_start + left_newlines;
                self.global_line_offset = newlines;
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
    iter: Chars<'a>,
    parse_contents: bool,
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
        let iter = Chars::new(n);

        Self::from_raw(iter)
    }

    const fn from_raw(iter: Chars<'a>) -> Self {
        Self {
            iter,
            parse_contents: true,
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
        let line_number = self.iter.newlines_consumed();
        let character_offset = self.iter.characters_consumed();

        let mut met_nl = false;
        let (mut contents, mut length) = (String::new(), 0);
        for c in self.iter.by_ref() {
            length += 1;
            if c == '\n' {
                length -= 1;
                met_nl = true;
                break;
            }

            if self.parse_contents {
                contents.push(c);
            }
        }

        if length == 0 && !met_nl {
            return None;
        }

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
        self.iter.skip_lines(n);
        self.parse_contents = parse_setting;
        self.next()
    }
}

#[cfg(test)]
mod tests {

    use crate::{Chars, Node, Rope};

    fn example_rope() -> Rope {
        let m = Node::Leaf(Box::from("s"));
        let n = Node::Leaf(Box::from(" Simon"));
        let j = Node::Leaf(Box::from("na"));
        let k = Node::Leaf(Box::from("me i"));
        let g = Node::Value {
            left_len: 2,
            left_newlines: 0,
            l: Some(Box::new(j)),
            r: Some(Box::new(k)),
        };
        let h = Node::Value {
            left_len: 1,
            left_newlines: 0,
            l: Some(Box::new(m)),
            r: Some(Box::new(n)),
        };
        let e = Node::Leaf(Box::from("Hello "));
        let f = Node::Leaf(Box::from("my "));
        let c = Node::Value {
            left_len: 6,
            left_newlines: 0,
            l: Some(Box::new(e)),
            r: Some(Box::new(f)),
        };
        let d = Node::Value {
            left_len: 6,
            left_newlines: 0,
            l: Some(Box::new(g)),
            r: Some(Box::new(h)),
        };
        let b = Node::Value {
            left_len: 9,
            left_newlines: 0,
            l: Some(Box::new(c)),
            r: Some(Box::new(d)),
        };
        let a = Node::Value {
            left_len: 22,
            left_newlines: 0,
            l: Some(Box::new(b)),
            r: None,
        };
        Rope { root: Box::new(a) }
    }

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

        let mut rope = Rope::from("\n");
        rope.insert(0, &"c");
        rope.insert(2, &"c");
        let mut lines = rope.lines();
        assert_eq!(lines.nth(1).unwrap().length, 1);

        let mut rope = Rope::from("\nHe");
        rope.insert(0, &"c");
        assert_eq!(rope.total_lines(), 1);
        let mut lines = rope.lines();
        assert_eq!(lines.nth(1).unwrap().length, 2);

        let rope = Rope::from("c\nHe");
        assert_eq!(rope.total_lines(), 1);
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

    #[test]
    fn chars_skip() {
        let rope = example_rope();
        let mut chars = rope.chars().skip(7);
        assert_eq!(chars.next(), Some('y'));
    }
}
