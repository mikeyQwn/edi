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

#[derive(Debug)]
pub struct Rope {
    root: Box<Node>,
}

impl Rope {
    pub fn new() -> Self {
        Self {
            root: Box::new(Node::Leaf(Box::from(""))),
        }
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        Iter::new(self).flat_map(|s| s.chars())
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
}
