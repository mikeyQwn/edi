use std::{
    iter::{Enumerate, FlatMap, Peekable, Skip, Take},
    ops::Range,
};

use super::{Node, Rope};

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

pub struct Lines<'a> {
    iter: Peekable<Enumerate<Chars<'a>>>,
    parse_contents: bool,
    seen_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineInfo {
    pub line_number: usize,
    pub character_offset: usize,
    pub contents: String,
}

impl<'a> Lines<'a> {
    #[must_use]
    pub fn new(n: &'a Node) -> Self {
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
        let contents = if self.parse_contents {
            self.iter
                .by_ref()
                .take_while(|&(_, char)| char != '\n')
                .map(|(_, char)| char)
                .collect()
        } else {
            // Consume line contents
            self.iter
                .by_ref()
                .take_while(|&(_, char)| char != '\n')
                .for_each(|(_, _)| {});
            String::new()
        };

        self.seen_count += 1;

        Some(LineInfo {
            line_number,
            character_offset,
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
