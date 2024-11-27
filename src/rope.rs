enum Node {
    Leaf(Box<str>),
    Regular {
        val: usize,
        l: Option<Box<Node>>,
        r: Option<Box<Node>>,
    },
}

pub struct Rope {
    root: Node,
}

impl Rope {
    pub fn new() -> Self {
        Self {
            root: Node::Leaf(Box::from("")),
        }
    }

    pub fn chars(&self) -> impl Iterator<Item = char> {
        std::iter::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Rope;

    #[test]
    fn general() {
        let r = Rope::new();
        let s = r.chars().collect::<String>();
        assert_eq!(s, "");
    }
}
