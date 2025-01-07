pub mod iter;

use std::ops::{Range, RangeBounds};

use iter::{Chars, LineInfo, Lines, Substring};

// A node in the rope binary tree.
#[derive(Debug)]
enum Node {
    // A leaf node contains an immutable string.
    // Any operation that modifies the contained string should create new leaf nodes.
    Leaf(Box<str>),
    // A value node contains a cumulative length of the left subtree leaf nodes' lengths.
    Value {
        // The value of the node is the cumulative length of the left subtree leaf nodes' lengths
        val: usize,
        // The left child of the node
        l: Option<Box<Node>>,
        // The right child of the node
        r: Option<Box<Node>>,
    },
}

impl Node {
    // Returns the weight of the node.
    pub fn weight(&self) -> usize {
        match self {
            Node::Leaf(s) => s.chars().count(),
            Node::Value { val, .. } => *val,
        }
    }

    // Returns the weight of the node and all its children
    // The full weight of the root node is the total number of characters in the rope.
    pub fn full_weight(&self) -> usize {
        match self {
            Node::Leaf(s) => s.chars().count(),
            Node::Value { val, r, .. } => {
                let r_weight = r.as_ref().map_or(0, |ri| ri.full_weight());
                val + r_weight
            }
        }
    }

    // Returns a reference to the right child node, if any
    pub fn right(&self) -> Option<&Node> {
        match self {
            Node::Leaf(_) => None,
            Node::Value { r, .. } => r.as_deref(),
        }
    }

    // Returns a reference to the left child node, if any
    pub fn left(&self) -> Option<&Node> {
        match self {
            Node::Leaf(_) => None,
            Node::Value { l, .. } => l.as_deref(),
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::Leaf(Box::default())
    }
}

// A rope data structure
#[derive(Debug)]
pub struct Rope {
    root: Box<Node>,
}

impl Rope {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // Returns the depth of the tree
    #[must_use]
    pub fn depth(&self) -> usize {
        Self::depth_inner(&self.root)
    }

    fn depth_inner(node: &Node) -> usize {
        match node {
            Node::Leaf(_) => 1,
            Node::Value { l, r, .. } => {
                let l_depth = l.as_ref().map_or(0, |le| Self::depth_inner(le));
                let r_depth = r.as_ref().map_or(0, |ri| Self::depth_inner(ri));
                1 + l_depth.max(r_depth)
            }
        }
    }

    // Concatenates `self` with `other`
    pub fn concat(&mut self, mut other: Rope) {
        // An edge case where the current rope is empty to avoid keeping empty nodes in the tree
        if self.root.weight() == 0 {
            let _ = std::mem::replace(&mut self.root, other.root);
            return;
        }

        let new_root = Node::Value {
            val: self.len(),
            l: Some(std::mem::take(&mut self.root)),
            r: Some(std::mem::take(&mut other.root)),
        };

        self.root = Box::new(new_root);
    }

    // Returns the length in characters of the string represented by the rope.
    pub fn len(&self) -> usize {
        self.root.full_weight()
    }

    // Removes characters in the given range from the rope
    pub fn delete(&mut self, range: impl std::ops::RangeBounds<usize>) {
        let range = self.normalize_range(range);
        let (mut left, mut right) = self.split(range.start);
        let (_, right) = right.split(range.end - range.start);
        left.concat(right);
        *self = left;
    }

    fn weight(&self) -> usize {
        self.root.weight()
    }

    fn is_balanced(&self) -> bool {
        static FIB: [usize; 64] = {
            let mut fib = [0; 64];
            fib[0] = 0;
            fib[1] = 1;
            let mut i = 2;
            while i < 64 {
                fib[i] = fib[i - 1] + fib[i - 2];
                i += 1;
            }
            fib
        };

        let depth = self.depth();
        if depth >= FIB.len() {
            return false;
        }

        FIB[depth + 2] <= self.weight()
    }

    fn merge_range(leaves: &mut [Node], range: std::ops::Range<usize>) -> Node {
        let len = range.end - range.start;
        if len == 1 {
            return std::mem::take(&mut leaves[range.start]);
        }
        if len == 2 {
            let Node::Leaf(ref l) = leaves[range.start] else {
                unreachable!("all nodes passed to merge_range should be of type leaf");
            };

            return Node::Value {
                val: l.chars().count(),
                l: Some(Box::new(std::mem::take(&mut leaves[range.start]))),
                r: Some(Box::new(std::mem::take(&mut leaves[range.start + 1]))),
            };
        }

        let mid = range.start + len / 2;
        let left = Self::merge_range(leaves, range.start..mid);
        let left_weight = left.full_weight();
        let right = Self::merge_range(leaves, mid..range.end);

        Node::Value {
            val: left_weight,
            l: Some(Box::new(left)),
            r: Some(Box::new(right)),
        }
    }

    fn rebalance(&mut self) {
        if self.is_balanced() {
            return;
        }

        let mut leaves = self.get_leaves();
        let len = leaves.len();
        self.root = Box::new(Self::merge_range(&mut leaves, 0..len));
    }

    fn get_leaves(&mut self) -> Vec<Node> {
        let mut leaves: Vec<Node> = Vec::new();
        let root = *std::mem::take(&mut self.root);
        Self::get_leaves_inner(root, &mut leaves);

        leaves
    }

    fn get_leaves_inner(node: Node, leaves: &mut Vec<Node>) {
        match node {
            Node::Leaf(_) => leaves.push(node),
            Node::Value { l, r, .. } => {
                if let Some(l) = l {
                    Self::get_leaves_inner(*l, leaves);
                }
                if let Some(r) = r {
                    Self::get_leaves_inner(*r, leaves);
                }
            }
        }
    }

    // Returns nth character of the string representation of the rope
    pub fn get(&self, n: usize) -> Option<char> {
        Self::get_inner(&self.root, n)
    }

    fn get_inner(node: &Node, n: usize) -> Option<char> {
        match node {
            Node::Leaf(s) => s.chars().nth(n),
            Node::Value { val, l, r } => {
                if n < *val {
                    Self::get_inner(l.as_ref()?, n)
                } else {
                    Self::get_inner(r.as_ref()?, n - val)
                }
            }
        }
    }

    // Splits the rope in two at the character index
    pub fn split(&mut self, idx: usize) -> (Rope, Rope) {
        let (l_node, r_node) = Self::split_inner(std::mem::take(&mut self.root), idx);

        let mut left = Rope { root: l_node };
        left.rebalance();

        let mut right = Rope { root: r_node };
        right.rebalance();

        (left, right)
    }

    fn split_inner(node: Node, idx: usize) -> (Box<Node>, Box<Node>) {
        match node {
            Node::Leaf(s) => {
                let left = Box::new(Node::Leaf(s[..idx].into()));
                let right = Box::new(Node::Leaf(s[idx..].into()));
                (left, right)
            }
            Node::Value { val, l, r } => {
                if idx < val {
                    let (left, right) = Self::split_inner(*l.unwrap(), idx);
                    let right = Box::new(Node::Value {
                        val: val - idx,
                        l: Some(right),
                        r,
                    });

                    (left, right)
                } else {
                    let (left, right) = Self::split_inner(*r.unwrap(), idx - val);
                    let left = Box::new(Node::Value {
                        val,
                        l,
                        r: Some(left),
                    });

                    (left, right)
                }
            }
        }
    }

    pub fn insert(&mut self, idx: usize, s: &str) {
        if idx == 0 {
            self.prepend(s);
            return;
        }

        if idx == self.len() {
            self.concat(Rope::from(s.to_owned()));
            return;
        }

        let (mut left, right) = self.split(idx);
        left.concat(Rope::from(s.to_owned()));
        left.concat(right);
        *self = left;
    }

    fn prepend(&mut self, s: &str) {
        let mut new = Rope::from(s.to_owned());
        new.concat(std::mem::take(self));
        *self = new;
    }

    pub fn chars(&self) -> Chars<'_> {
        Chars::new(&self.root)
    }

    pub fn lines(&self) -> Lines<'_> {
        Lines::new(&self.root)
    }

    pub fn line(&self, n: usize) -> Option<LineInfo> {
        Lines::new(&self.root).nth(n)
    }

    pub fn line_info(&self, n: usize) -> Option<LineInfo> {
        Lines::new(&self.root).parse_contents(false).nth(n)
    }

    pub fn prev_line_start(&self, idx: usize) -> Option<usize> {
        if idx == 0 {
            return None;
        }

        let mut it = self
            .chars()
            .enumerate()
            .take(idx)
            .filter(|&(_, c)| c == '\n')
            .peekable();

        let mut res = None;
        while let Some((pos, _)) = it.next() {
            if it.peek().is_some() {
                res = Some(pos + 1);
            }
        }

        Some(res.unwrap_or(0))
    }

    pub fn next_line_start(&self, idx: usize) -> Option<usize> {
        let mut it = self.chars().enumerate().skip(idx).peekable();
        it.by_ref().find(|&(_, c)| c == '\n');
        it.next().map(|(pos, _)| pos)
    }

    pub fn line_start(&self, n: usize) -> Option<usize> {
        self.chars()
            .enumerate()
            .filter(|&(_, c)| c == '\n')
            .nth(n)
            .map(|(idx, _)| idx + 1)
    }

    pub fn line_starts(&self) -> impl Iterator<Item = usize> + '_ {
        std::iter::once(0).chain(
            self.chars()
                .enumerate()
                .filter(|&(_, c)| c == '\n')
                .map(|(pos, _)| pos + 1),
        )
    }

    pub fn substr(&self, range: impl RangeBounds<usize>) -> Substring<'_> {
        let mut range = self.normalize_range(range);

        let (node, skipped) = Self::skip_to(&self.root, range.start);
        range.start -= skipped;
        range.end -= skipped;

        Substring::new(Chars::new(node), range)
    }

    fn normalize_range(&self, range: impl std::ops::RangeBounds<usize>) -> Range<usize> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&s) => s,
            std::ops::Bound::Excluded(&s) => s + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(&e) => e + 1,
            std::ops::Bound::Excluded(&e) => e,
            std::ops::Bound::Unbounded => self.len(),
        };

        start..end
    }

    fn skip_to<'a>(mut from: &Node, target: usize) -> (&Node, usize) {
        let mut skipped = 0;
        // Skip the left subtree if it is not included in the substring
        while let Node::Value { val, r, .. } = from {
            if *val >= target - skipped {
                break;
            }

            let Some(r) = r else {
                break;
            };

            from = r;
            skipped += val;
        }

        (from, skipped)
    }
}

impl From<String> for Rope {
    fn from(s: String) -> Self {
        let node = Node::Leaf(s.into_boxed_str());
        Rope {
            root: Box::new(node),
        }
    }
}

impl Default for Rope {
    fn default() -> Self {
        Self {
            root: Box::new(Node::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_rope() -> Rope {
        let m = Node::Leaf(Box::from("s"));
        let n = Node::Leaf(Box::from(" Simon"));
        let j = Node::Leaf(Box::from("na"));
        let k = Node::Leaf(Box::from("me i"));
        let g = Node::Value {
            val: 2,
            l: Some(Box::new(j)),
            r: Some(Box::new(k)),
        };
        let h = Node::Value {
            val: 1,
            l: Some(Box::new(m)),
            r: Some(Box::new(n)),
        };
        let e = Node::Leaf(Box::from("Hello "));
        let f = Node::Leaf(Box::from("my "));
        let c = Node::Value {
            val: 6,
            l: Some(Box::new(e)),
            r: Some(Box::new(f)),
        };
        let d = Node::Value {
            val: 6,
            l: Some(Box::new(g)),
            r: Some(Box::new(h)),
        };
        let b = Node::Value {
            val: 9,
            l: Some(Box::new(c)),
            r: Some(Box::new(d)),
        };
        let a = Node::Value {
            val: 22,
            l: Some(Box::new(b)),
            r: None,
        };
        Rope { root: Box::new(a) }
    }

    fn assert_correctness(r: &mut Rope, expected: &str) {
        assert_eq!(r.chars().collect::<String>(), expected);
        expected.chars().enumerate().for_each(|(i, c)| {
            assert_eq!(
                r.get(i),
                Some(c),
                "r: {}, e: {}, idx: {}\nr: {:#?}",
                r.chars().collect::<String>(),
                expected,
                i,
                r
            );
        });
        assert_eq!(r.chars().count(), r.len());
        if r.chars().all(|c| c.is_ascii()) {
            for start in 0..expected.len() {
                for end in start..expected.len() {
                    assert_eq!(
                        expected[start..end],
                        r.substr(start..end).collect::<String>()
                    )
                }
            }
        }
    }

    #[test]
    fn empty() {
        let r = Rope::new();
        let s = r.chars().collect::<String>();
        assert_eq!(s, "");
    }

    #[test]
    fn traversal() {
        let mut r = example_rope();
        let expected = "Hello my name is Simon".to_owned();
        assert_correctness(&mut r, &expected);
    }

    #[test]
    fn depth() {
        let mut r = example_rope();
        let expected = "Hello my name is Simon".to_owned();

        let mut leaves = r.get_leaves();
        let len = leaves.len();
        r.root = Box::new(Rope::merge_range(&mut leaves, 0..len));

        assert_correctness(&mut r, &expected);
    }

    #[test]
    fn concat() {
        let mut r = example_rope();
        let second = Rope::from(" and I like to eat pizza".to_owned());
        let expected = "Hello my name is Simon and I like to eat pizza".to_owned();

        r.concat(second);
        assert_correctness(&mut r, &expected);
    }

    #[test]
    fn like_string() {
        let cases = vec![
            (
                vec!["Hello ", "my ", "name ", "is ", "Simon"],
                "Hello my name is Simon",
            ),
            (
                vec![
                    "Hello ",
                    "my ",
                    "name ",
                    "is ",
                    "Simon",
                    " and I like to eat pizza",
                ],
                "Hello my name is Simon and I like to eat pizza",
            ),
            (vec!["", ""], ""),
            (vec!["", "a"], "a"),
            (vec!["a", ""], "a"),
            (vec!["a", "b"], "ab"),
            (vec!["a", "b", "c"], "abc"),
            (vec![" ", " ", " "], "   "),
        ];

        for (input, expected) in cases {
            let mut r = Rope::new();
            for s in input {
                r.concat(Rope::from(s.to_owned()));
            }

            assert_correctness(&mut r, &expected);
        }
    }

    #[test]
    fn split() {
        let mut r = example_rope();
        let expected = "Hello my name is Simon".to_owned();
        assert_correctness(&mut r, &expected);

        let (mut left, mut right) = r.split(5);
        assert_correctness(&mut left, "Hello");
        assert_correctness(&mut right, " my name is Simon");
    }

    #[test]
    fn split_and_concat() {
        let mut r = example_rope();
        let expected = "Hello my name is Simon".to_owned();
        assert_correctness(&mut r, &expected);

        let (mut left, mut right) = r.split(5);
        assert_correctness(&mut left, "Hello");
        assert_correctness(&mut right, " my name is Simon");

        left.concat(right);
        assert_correctness(&mut left, &expected);

        let mut r = Rope::from(String::new());
        r.insert(0, ":");
        assert_correctness(&mut r, ":");
        assert_eq!(r.len(), 1);
        r.insert(1, "w");
        assert_correctness(&mut r, ":w");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn insert() {
        let mut r = example_rope();
        r.insert(5, " woah");
        let expected = "Hello woah my name is Simon".to_owned();

        assert_correctness(&mut r, &expected);
    }

    #[test]
    fn delete() {
        let mut r = example_rope();
        let str = "Hello my name is Simon";
        r.delete(5..8);
        let expected: String = str
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if i >= 5 && i < 8 { None } else { Some(c) })
            .collect();

        assert_correctness(&mut r, &expected);
    }
}
