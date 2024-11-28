#[derive(Debug)]
enum Node {
    Leaf(Box<str>),
    Value {
        // The value of the node is the cumulative length of the left subtree leaf nodes' lengths
        val: usize,
        l: Option<Box<Node>>,
        r: Option<Box<Node>>,
    },
}

impl Default for Node {
    fn default() -> Self {
        Node::Leaf(Box::from(""))
    }
}

#[derive(Debug)]
pub struct Rope {
    root: Box<Node>,
}

impl Rope {
    pub fn new() -> Self {
        Self::default()
    }

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

    fn weight(&self) -> usize {
        match self.root.as_ref() {
            Node::Leaf(s) => s.len(),
            Node::Value { val, .. } => *val,
        }
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
                val: l.len(),
                l: Some(Box::new(std::mem::take(&mut leaves[range.start]))),
                r: Some(Box::new(std::mem::take(&mut leaves[range.start + 1]))),
            };
        }

        let mid = range.start + len / 2;
        let left = Self::merge_range(leaves, range.start..mid);
        let left_weight = match left {
            Node::Leaf(ref s) => s.len(),
            Node::Value { val, .. } => val,
        };
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

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        Iter::new(self).flat_map(|s| s.chars())
    }
}

impl Default for Rope {
    fn default() -> Self {
        Self {
            root: Box::new(Node::default()),
        }
    }
}

#[derive(Default)]
struct Iter<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iter<'a> {
    pub fn new(r: &'a Rope) -> Self {
        let it: Option<&Node> = Some(&r.root);
        let mut out = Self::default();
        out.push_left(it);

        out
    }

    fn push_left(&mut self, mut it: Option<&'a Node>) {
        while let Some(value) = it {
            self.stack.push(value);
            match value {
                Node::Value { l: Some(l), .. } => {
                    it = Some(l);
                }
                _ => it = None,
            }
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let string = match self.stack.pop() {
            Some(Node::Leaf(string)) => string,
            Some(_) => unreachable!("all leaf nodes should be of type leaf, as there is no way to create an iterator with invalid rope"),
            None => return None,
        };

        if self.stack.is_empty() {
            return Some(string);
        }
        // Take the right child of the current node and push all its left children onto the stack
        let Some(Node::Value { r: Some(r), .. }) = self.stack.pop() else {
            return Some(string);
        };
        self.stack.push(r);

        let Node::Value { l: it, .. } = r.as_ref() else {
            return Some(string);
        };

        let it = it.as_deref();
        self.push_left(it);

        Some(string)
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

    #[test]
    fn general() {
        let r = Rope::new();
        let s = r.chars().collect::<String>();
        assert_eq!(s, "");
    }

    #[test]
    fn traversal() {
        let r = example_rope();
        let expected = "Hello my name is Simon".to_owned();
        assert_eq!(r.chars().collect::<String>(), expected)
    }

    #[test]
    fn depth() {
        let mut r = example_rope();
        let expected = "Hello my name is Simon".to_owned();

        let mut leaves = r.get_leaves();
        let len = leaves.len();
        r.root = Box::new(Rope::merge_range(&mut leaves, 0..len));

        assert_eq!(r.chars().collect::<String>(), expected);
    }
}
