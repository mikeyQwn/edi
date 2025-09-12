//! A data structure suited for frequent write operations

#![deny(missing_docs)]

#[cfg(test)]
use criterion as _;

mod info;
mod leaf;
mod string;
mod value;

// pub mod iter;
pub mod node;

use std::fmt::Debug;

// use iter::{Chars, LineInfo, Lines, Substring};
use node::Node;

/// Rope data structure. It is optimized for frequent modification
#[derive(Debug)]
pub struct Rope {
    root: Node,
}

impl Rope {
    /// Initiates an empty `Rope`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the character length of the string represented by the rope
    #[must_use]
    pub fn len(&self) -> usize {
        self.root.weight()
    }

    /// Returns the number of lines in the rope
    #[must_use]
    pub fn total_lines(&self) -> usize {
        self.root.newlines()
    }

    /// Returns an ASCII tree representation of the rope's node structure
    #[must_use]
    pub fn to_ascii_tree(&self) -> String {
        format!(
            "Rope [{} chars, {} lines]\n{}",
            self.len(),
            self.total_lines(),
            self.root.to_ascii_tree()
        )
    }

    /// Returns the depth of `Ropes`'s `Node` tree
    #[must_use]
    pub fn depth(&self) -> usize {
        self.root.depth()
    }
}

// impl Rope {
//     /// Concatenates `self` with `other`. The string representation becomes exactly `self` + `other`
//     pub fn concat(&mut self, mut other: Rope) {
//         // An edge case where the current rope is empty to avoid keeping empty nodes in the tree
//         if self.root.weight() == 0 {
//             let _ = std::mem::replace(&mut self.root, other.root);
//             return;
//         }
//
//         let new_root = Node::Value {
//             left_len: self.len(),
//             left_newlines: self.total_lines(),
//             l: Some(std::mem::take(&mut self.root)),
//             r: Some(std::mem::take(&mut other.root)),
//         };
//
//         self.root = Box::new(new_root);
//         self.validate_newlines();
//     }
//
//     /// Validates that all `left_newlines` fields in the tree correctly represent
//     /// the number of newlines in their left subtrees.
//     /// Panics if any inconsistency is found.
//     pub fn validate_newlines(&self) {
//         #[cfg(debug_assertions)]
//         {
//             Rope::validate_newlines_inner(&self.root);
//         };
//     }
//
//     #[allow(unused)]
//     fn validate_newlines_inner(node: &Node) -> usize {
//         match node {
//             Node::Leaf { newlines, .. } => *newlines,
//             Node::Value {
//                 left_len: _,
//                 left_newlines,
//                 l,
//                 r,
//             } => {
//                 let left_newlines_actual = l
//                     .as_ref()
//                     .map_or(0, |left| Self::validate_newlines_inner(left));
//                 let right_newlines_actual = r
//                     .as_ref()
//                     .map_or(0, |right| Self::validate_newlines_inner(right));
//
//                 assert_eq!(
//                     *left_newlines, left_newlines_actual,
//                     "left_newlines validation failed: stored={left_newlines}, actual={left_newlines_actual}",
//                 );
//
//                 left_newlines_actual + right_newlines_actual
//             }
//         }
//     }
//
//
//
//     /// Returns `true` if the `Rope` contains no characters
//     #[must_use]
//     pub fn is_empty(&self) -> bool {
//         self.len() == 0
//     }
//
//     /// Removes substring in the given character range from the `Rope`
//     pub fn delete(&mut self, range: impl std::ops::RangeBounds<usize>) {
//         let range = self.normalize_range(range);
//         let (mut left, mut right) = self.split(range.start);
//         let (_, right) = right.split(range.end - range.start);
//         left.concat(right);
//         *self = left;
//     }
//
//     fn weight(&self) -> usize {
//         self.root.weight()
//     }
//
//     fn is_balanced(&self) -> bool {
//         static FIB: [usize; 64] = {
//             let mut fib = [0; 64];
//             fib[0] = 0;
//             fib[1] = 1;
//             let mut i = 2;
//             while i < 64 {
//                 fib[i] = fib[i - 1] + fib[i - 2];
//                 i += 1;
//             }
//             fib
//         };
//
//         let depth = self.depth();
//         if depth >= FIB.len() {
//             return false;
//         }
//
//         FIB[depth + 2] <= self.weight()
//     }
//
//     fn merge_range(leaves: &mut [Node], range: std::ops::Range<usize>) -> Node {
//         let len = range.end - range.start;
//         if len == 1 {
//             return std::mem::take(&mut leaves[range.start]);
//         }
//         if len == 2 {
//             let Node::Leaf {
//                 char_len, newlines, ..
//             } = &leaves[range.start]
//             else {
//                 unreachable!("all nodes passed to merge_range should be of type leaf");
//             };
//
//             return Node::Value {
//                 left_len: *char_len,
//                 left_newlines: *newlines,
//                 l: Some(Box::new(std::mem::take(&mut leaves[range.start]))),
//                 r: Some(Box::new(std::mem::take(&mut leaves[range.start + 1]))),
//             };
//         }
//
//         let mid = range.start + len / 2;
//         let left = Self::merge_range(leaves, range.start..mid);
//         let left_weight = left.full_weight();
//         let left_newlines = left.full_newlines();
//         let right = Self::merge_range(leaves, mid..range.end);
//
//         Node::Value {
//             left_len: left_weight,
//             left_newlines,
//             l: Some(Box::new(left)),
//             r: Some(Box::new(right)),
//         }
//     }
//
//     fn rebalance(&mut self) {
//         if self.is_balanced() {
//             return;
//         }
//
//         let mut leaves = self.get_leaves();
//         let len = leaves.len();
//         self.root = Box::new(Self::merge_range(&mut leaves, 0..len));
//     }
//
//     fn get_leaves(&mut self) -> Vec<Node> {
//         let mut leaves: Vec<Node> = Vec::new();
//         let root = *std::mem::take(&mut self.root);
//         Self::get_leaves_inner(root, &mut leaves);
//
//         leaves
//     }
//
//     fn get_leaves_inner(node: Node, leaves: &mut Vec<Node>) {
//         match node {
//             Node::Leaf { .. } => leaves.push(node),
//             Node::Value { l, r, .. } => {
//                 if let Some(l) = l {
//                     Self::get_leaves_inner(*l, leaves);
//                 }
//                 if let Some(r) = r {
//                     Self::get_leaves_inner(*r, leaves);
//                 }
//             }
//         }
//     }
//
//     /// Returns nth character of the string representation of the rope
//     #[must_use]
//     pub fn get(&self, n: usize) -> Option<char> {
//         Self::get_inner(&self.root, n)
//     }
//
//     fn get_inner(node: &Node, n: usize) -> Option<char> {
//         match node {
//             Node::Leaf { value, .. } => value.chars().nth(n),
//             Node::Value {
//                 left_len: val,
//                 l,
//                 r,
//                 ..
//             } => {
//                 if n < *val {
//                     Self::get_inner(l.as_ref()?, n)
//                 } else {
//                     Self::get_inner(r.as_ref()?, n - val)
//                 }
//             }
//         }
//     }
//
//     /// Splits the rope in two at the character index
//     pub fn split(&mut self, idx: usize) -> (Rope, Rope) {
//         let (l_node, r_node) = Self::split_inner(std::mem::take(&mut self.root), idx);
//
//         let mut left = Rope { root: l_node };
//         left.rebalance();
//
//         let mut right = Rope { root: r_node };
//         right.rebalance();
//
//         (left, right)
//     }
//
//     fn split_inner(node: Node, idx: usize) -> (Box<Node>, Box<Node>) {
//         match node {
//             Node::Leaf { value, .. } => {
//                 let left = Box::new(Node::new_leaf(value[..idx].into()));
//                 let right = Box::new(Node::new_leaf(value[idx..].into()));
//                 (left, right)
//             }
//             Node::Value {
//                 left_len: val,
//                 l,
//                 r,
//                 ..
//             } => {
//                 if idx < val {
//                     let (left, right) = Self::split_inner(*l.unwrap(), idx);
//                     let right_newlines = right.full_newlines();
//                     let right = Box::new(Node::Value {
//                         left_len: val - idx,
//                         left_newlines: right_newlines,
//                         l: Some(right),
//                         r,
//                     });
//
//                     (left, right)
//                 } else {
//                     let (left, right) = Self::split_inner(*r.unwrap(), idx - val);
//                     let left = Box::new(Node::Value {
//                         left_len: val,
//                         left_newlines: l.as_deref().map_or(0, Node::full_newlines),
//                         l,
//                         r: Some(left),
//                     });
//
//                     (left, right)
//                 }
//             }
//         }
//     }
//
//     /// Inserts `s` at `idx` character position
//     pub fn insert(&mut self, idx: usize, s: &str) {
//         if idx == 0 {
//             self.prepend(s);
//             return;
//         }
//
//         if idx == self.len() {
//             self.concat(Rope::from(s));
//             return;
//         }
//
//         let (mut left, right) = self.split(idx);
//         left.concat(Rope::from(s));
//         left.concat(right);
//         *self = left;
//     }
//
//     fn prepend(&mut self, s: &str) {
//         let mut new = Rope::from(s);
//         new.concat(std::mem::take(self));
//         *self = new;
//     }
//
//     /// Returns iterator over represented string's characters
//     #[must_use]
//     pub fn chars(&self) -> Chars<'_> {
//         Chars::new(&self.root)
//     }
//
//     /// Returns iterator over represented string's lines
//     ///
//     /// The iterator yeilds not just string representations, but line's character offset, number
//     /// and length
//     #[must_use]
//     pub fn lines(&self) -> Lines<'_> {
//         Lines::new(&self.root)
//     }
//
//     /// Returns `n`th line information, including string representation
//     ///
//     /// If string representation is not needed, consider using `line_info` instead, to avoid
//     /// allocation
//     #[must_use]
//     pub fn line(&self, n: usize) -> Option<LineInfo> {
//         Lines::new(&self.root).nth(n)
//     }
//
//     /// Returns `n`th line information, excluding string representation
//     ///
//     /// If string representation is needed, use `line` instead
//     #[must_use]
//     pub fn line_info(&self, n: usize) -> Option<LineInfo> {
//         Lines::new(&self.root).parse_contents(false).nth(n)
//     }
//
//     /// Returns iterator over represented string's substring
//     ///
//     /// Functionally is the same as `self.chars().skip(range.start).take(range.len())`, but
//     /// optimized to skip `Node`s that don't include the range
//     #[must_use]
//     pub fn substr(&self, range: impl RangeBounds<usize>) -> Substring<'_> {
//         let range = self.normalize_range(range);
//         Substring::new(Chars::new(&self.root), range)
//     }
//
//     /// Returns number of the line containing given index
//     #[must_use]
//     pub fn line_of_index(&self, index: usize) -> usize {
//         let (node, skipped, lines_skipped) = Self::skip_to(&self.root, index);
//         let to_parse = index - skipped;
//
//         lines_skipped
//             + Chars::new(node)
//                 .take(to_parse)
//                 .filter(|&c| c == '\n')
//                 .count()
//     }
//
//     /// Returns the line start index
//     #[must_use]
//     pub fn index_of_line(&self, line: usize) -> usize {
//         self.root.index_of_line(line)
//     }
//
//     /// Converts a string into the rope. The number of bytes in a rope leaf may never exceed
//     /// `chunk_size` + 3
//     #[must_use]
//     pub fn from_str_chunked(s: &str, chunk_size: usize) -> Rope {
//         let mut rope = Rope::default();
//         let mut offset = 0;
//         while offset < s.len() {
//             let mut end = (offset + chunk_size).min(s.len());
//             while !s.is_char_boundary(end) {
//                 end += 1;
//                 // TODO: handle this case
//                 assert!(offset < s.len(), "invalid utf-8 encoded string");
//             }
//
//             rope.concat(Rope {
//                 root: Box::new(Node::new_leaf(&s[offset..end])),
//             });
//             offset = end;
//         }
//
//         #[cfg(debug_assertions)]
//         rope.validate_newlines();
//
//         rope
//     }
//
//     fn normalize_range(&self, range: impl std::ops::RangeBounds<usize>) -> Range<usize> {
//         let start = match range.start_bound() {
//             std::ops::Bound::Included(&s) => s,
//             std::ops::Bound::Excluded(&s) => s + 1,
//             std::ops::Bound::Unbounded => 0,
//         };
//
//         let end = match range.end_bound() {
//             std::ops::Bound::Included(&e) => e + 1,
//             std::ops::Bound::Excluded(&e) => e,
//             std::ops::Bound::Unbounded => self.len(),
//         };
//
//         start..end
//     }
//
//     fn skip_to(mut from: &Node, target: usize) -> (&Node, usize, usize) {
//         let mut skipped = 0;
//         let mut skipped_lines = 0;
//         // Skip the left subtree if it is not included in the substring
//         while let Node::Value {
//             left_len: val,
//             r,
//             left_newlines,
//             ..
//         } = from
//         {
//             if *val >= target - skipped {
//                 break;
//             }
//
//             let Some(r) = r else {
//                 break;
//             };
//
//             from = r;
//             skipped += val;
//             skipped_lines += left_newlines;
//         }
//
//         (from, skipped, skipped_lines)
//     }
// }
//
// impl From<&str> for Rope {
//     fn from(s: &str) -> Self {
//         const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;
//         Self::from_str_chunked(s, DEFAULT_CHUNK_SIZE)
//     }
// }
//
impl Default for Rope {
    fn default() -> Self {
        Self {
            root: Node::empty_value(),
        }
    }
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     fn example_rope() -> Rope {
//         let m = Node::new_leaf("s");
//         let n = Node::new_leaf(" Simon");
//         let j = Node::new_leaf("na");
//         let k = Node::new_leaf("me i");
//         let g = Node::Value {
//             left_len: 2,
//             left_newlines: 0,
//             l: Some(Box::new(j)),
//             r: Some(Box::new(k)),
//         };
//         let h = Node::Value {
//             left_len: 1,
//             left_newlines: 0,
//             l: Some(Box::new(m)),
//             r: Some(Box::new(n)),
//         };
//         let e = Node::new_leaf("Hello ");
//         let f = Node::new_leaf("my ");
//         let c = Node::Value {
//             left_len: 6,
//             left_newlines: 0,
//             l: Some(Box::new(e)),
//             r: Some(Box::new(f)),
//         };
//         let d = Node::Value {
//             left_len: 6,
//             left_newlines: 0,
//             l: Some(Box::new(g)),
//             r: Some(Box::new(h)),
//         };
//         let b = Node::Value {
//             left_len: 9,
//             left_newlines: 0,
//             l: Some(Box::new(c)),
//             r: Some(Box::new(d)),
//         };
//         let a = Node::Value {
//             left_len: 22,
//             left_newlines: 0,
//             l: Some(Box::new(b)),
//             r: None,
//         };
//         Rope { root: Box::new(a) }
//     }
//
//     fn assert_correctness(r: &mut Rope, expected: &str) {
//         assert_eq!(r.chars().collect::<String>(), expected);
//         expected.chars().enumerate().for_each(|(i, c)| {
//             assert_eq!(
//                 r.get(i),
//                 Some(c),
//                 "r: {}, e: {}, idx: {}\nr: {:#?}",
//                 r.chars().collect::<String>(),
//                 expected,
//                 i,
//                 r
//             );
//         });
//         assert_eq!(r.chars().count(), r.len());
//         if r.chars().all(|c| c.is_ascii()) {
//             for start in 0..expected.len() {
//                 for end in start..expected.len() {
//                     assert_eq!(
//                         expected[start..end],
//                         r.substr(start..end).collect::<String>(),
//                         "substring: {}, start: {start}, end: {end}",
//                         r.chars().collect::<String>(),
//                     )
//                 }
//             }
//         }
//         assert_eq!(r.lines().count(), expected.lines().count());
//         r.validate_newlines();
//     }
//
//     #[test]
//     fn empty() {
//         let r = Rope::new();
//         let s = r.chars().collect::<String>();
//         assert_eq!(s, "");
//     }
//
//     #[test]
//     fn traversal() {
//         let mut r = example_rope();
//         let expected = "Hello my name is Simon".to_owned();
//         assert_correctness(&mut r, &expected);
//     }
//
//     #[test]
//     fn depth() {
//         let mut r = example_rope();
//         let expected = "Hello my name is Simon".to_owned();
//
//         let mut leaves = r.get_leaves();
//         let len = leaves.len();
//         r.root = Box::new(Rope::merge_range(&mut leaves, 0..len));
//
//         assert_correctness(&mut r, &expected);
//     }
//
//     #[test]
//     fn concat() {
//         let mut r = example_rope();
//         let second = Rope::from(" and I like to eat pizza");
//         let expected = "Hello my name is Simon and I like to eat pizza";
//
//         r.concat(second);
//         assert_correctness(&mut r, &expected);
//     }
//
//     #[test]
//     fn like_string() {
//         let cases = vec![
//             (
//                 vec!["Hello ", "my ", "name ", "is ", "Simon"],
//                 "Hello my name is Simon",
//             ),
//             (
//                 vec![
//                     "Hello ",
//                     "my ",
//                     "name ",
//                     "is ",
//                     "Simon",
//                     " and I like to eat pizza",
//                 ],
//                 "Hello my name is Simon and I like to eat pizza",
//             ),
//             (vec!["", ""], ""),
//             (vec!["", "a"], "a"),
//             (vec!["a", ""], "a"),
//             (vec!["a", "b"], "ab"),
//             (vec!["a", "b", "c"], "abc"),
//             (vec![" ", " ", " "], "   "),
//         ];
//
//         for (input, expected) in cases {
//             let mut r = Rope::new();
//             for s in input {
//                 r.concat(Rope::from(s));
//             }
//
//             assert_correctness(&mut r, &expected);
//         }
//     }
//
//     #[test]
//     fn split() {
//         let mut r = example_rope();
//         let expected = "Hello my name is Simon".to_owned();
//         assert_correctness(&mut r, &expected);
//
//         let (mut left, mut right) = r.split(5);
//         assert_correctness(&mut left, "Hello");
//         assert_correctness(&mut right, " my name is Simon");
//     }
//
//     #[test]
//     fn split_and_concat() {
//         let mut r = example_rope();
//         let expected = "Hello my name is Simon".to_owned();
//         assert_correctness(&mut r, &expected);
//
//         let (mut left, mut right) = r.split(5);
//         assert_correctness(&mut left, "Hello");
//         assert_correctness(&mut right, " my name is Simon");
//
//         left.concat(right);
//         assert_correctness(&mut left, &expected);
//
//         let mut r = Rope::from("");
//         r.insert(0, ":");
//         assert_correctness(&mut r, ":");
//         assert_eq!(r.len(), 1);
//         r.insert(1, "w");
//         assert_correctness(&mut r, ":w");
//         assert_eq!(r.len(), 2);
//     }
//
//     #[test]
//     fn insert() {
//         let mut r = example_rope();
//         r.insert(5, " woah");
//         let expected = "Hello woah my name is Simon".to_owned();
//
//         assert_correctness(&mut r, &expected);
//     }
//
//     #[test]
//     fn delete() {
//         let mut r = example_rope();
//         let str = "Hello my name is Simon";
//         r.delete(5..8);
//         let expected: String = str
//             .chars()
//             .enumerate()
//             .filter_map(|(i, c)| if i >= 5 && i < 8 { None } else { Some(c) })
//             .collect();
//
//         assert_correctness(&mut r, &expected);
//     }
//
//     #[test]
//     fn weights_correctness() {
//         let r = example_rope();
//         assert_eq!(r.root.weight(), 22);
//         assert_eq!(r.root.full_weight(), 22);
//
//         if let Some(left) = r.root.left() {
//             assert_eq!(left.weight(), 9);
//             assert_eq!(left.full_weight(), 22);
//
//             if let Some(left_left) = left.left() {
//                 assert_eq!(left_left.weight(), 6);
//                 assert_eq!(left_left.full_weight(), 9);
//             }
//
//             if let Some(left_right) = left.right() {
//                 assert_eq!(left_right.weight(), 6);
//                 assert_eq!(left_right.full_weight(), 13);
//             }
//         }
//     }
//
//     #[test]
//     fn line_counting() {
//         let r = Rope::from("Hello\nworld\nthis\nis\na\ntest");
//
//         assert_eq!(r.total_lines(), 5);
//
//         assert_eq!(r.line_of_index(0), 0);
//         assert_eq!(r.line_of_index(5), 0);
//         assert_eq!(r.line_of_index(6), 1);
//         assert_eq!(r.line_of_index(11), 1);
//         assert_eq!(r.line_of_index(12), 2);
//
//         assert_eq!(r.index_of_line(0), 0);
//         assert_eq!(r.index_of_line(1), 6);
//         assert_eq!(r.index_of_line(2), 12);
//
//         let r = Rope::from("\n\n\n");
//         assert_eq!(r.total_lines(), 3);
//         assert_eq!(r.line_of_index(0), 0);
//         assert_eq!(r.line_of_index(1), 1);
//         assert_eq!(r.line_of_index(2), 2);
//         assert_eq!(r.index_of_line(0), 0);
//         assert_eq!(r.index_of_line(1), 1);
//         assert_eq!(r.index_of_line(2), 2);
//
//         let mut r = Rope::from("\nHe");
//         r.insert(0, "c");
//         assert_eq!(r.total_lines(), 1);
//     }
//
//     #[test]
//     fn line_counting_complex() {
//         let text = "First line\nSecond line\n\nFourth line\n";
//         let r = Rope::from(text);
//
//         assert_eq!(r.total_lines(), 4);
//
//         assert_eq!(r.line_of_index(0), 0); // 'F' in first line
//         assert_eq!(r.line_of_index(10), 0); // '\n' at end of first line
//         assert_eq!(r.line_of_index(11), 1); // 'S' in second line
//         assert_eq!(r.line_of_index(22), 1); // '\n' at end of second line
//         assert_eq!(r.line_of_index(23), 2); // '\n' (empty third line)
//         assert_eq!(r.line_of_index(24), 3); // 'F' in fourth line
//         assert_eq!(r.line_of_index(34), 3); // '\n' at end of fourth line
//     }
//
//     #[test]
//     fn weights_after_operations() {
//         let mut r = Rope::new();
//         assert_eq!(r.weight(), 0);
//         assert_eq!(r.len(), 0);
//
//         r.insert(0, "hello");
//         assert_eq!(r.weight(), 5);
//         assert_eq!(r.len(), 5);
//
//         r.insert(5, " world");
//
//         let (left, right) = r.split(5);
//         assert_eq!(left.weight(), 5);
//         assert_eq!(left.len(), 5);
//         assert_eq!(right.weight(), 6);
//         assert_eq!(right.len(), 6);
//     }
//
//     #[test]
//     fn line_counting_after_operations() {
//         let mut r = Rope::from("line1\nline2");
//         assert_eq!(r.total_lines(), 1);
//
//         r.insert(11, "\nline3");
//         assert_eq!(r.total_lines(), 2);
//
//         r.insert(0, "line0\n");
//         assert_eq!(r.total_lines(), 3);
//
//         let (mut left, right) = r.split(6);
//         assert_eq!(left.total_lines(), 1);
//         assert_eq!(right.total_lines(), 2);
//
//         left.concat(right);
//         assert_eq!(left.total_lines(), 3);
//
//         left.delete(5..6);
//         assert_eq!(left.total_lines(), 2);
//     }
// }
