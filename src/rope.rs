#[derive(Debug)]
enum Node {
    Leaf(Box<str>),
    Regular {
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
        let mut stack: Vec<&Node> = Vec::new();
        let mut it: Option<&Node> = Some(&self.root);
        while let Some(value) = it {
            stack.push(value);
            match value {
                Node::Regular {
                    val: _,
                    l: Some(l),
                    r: _,
                } => {
                    it = Some(l);
                }
                _ => it = None,
            }
        }

        Iter { stack }.flat_map(|s| s.chars())
    }
}

struct Iter<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let string = match self.stack.pop() {
            Some(Node::Leaf(string)) => string,
            Some(_) => unreachable!("all leaf nodes should be of type leaf"),
            None => return None,
        };

        let Some(Node::Regular {
            l: _,
            r: Some(r),
            val: _,
        }) = self.stack.pop()
        else {
            return Some(string);
        };

        self.stack.push(r);

        let Node::Regular {
            l: it,
            r: _,
            val: _,
        } = r.as_ref()
        else {
            return Some(string);
        };

        let mut it = it.as_deref();

        while let Some(value) = it {
            self.stack.push(value);
            match value {
                Node::Regular {
                    val: _,
                    l: Some(l),
                    r: _,
                } => {
                    it = Some(l);
                }
                _ => it = None,
            }
        }

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
        let g = Node::Regular {
            val: 2,
            l: Some(Box::new(j)),
            r: Some(Box::new(k)),
        };
        let h = Node::Regular {
            val: 1,
            l: Some(Box::new(m)),
            r: Some(Box::new(n)),
        };
        let e = Node::Leaf(Box::from("Hello "));
        let f = Node::Leaf(Box::from("my "));
        let c = Node::Regular {
            val: 6,
            l: Some(Box::new(e)),
            r: Some(Box::new(f)),
        };
        let d = Node::Regular {
            val: 6,
            l: Some(Box::new(g)),
            r: Some(Box::new(h)),
        };
        let b = Node::Regular {
            val: 9,
            l: Some(Box::new(c)),
            r: Some(Box::new(d)),
        };
        let a = Node::Regular {
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
