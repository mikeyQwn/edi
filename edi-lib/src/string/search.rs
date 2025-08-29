use std::iter::Peekable;

#[derive(Debug)]
pub struct Searcher<'a> {
    line: &'a str,
    offset: usize,
    rev: bool,

    allow_skip: bool,
}

impl<'a> Searcher<'a> {
    #[must_use]
    pub const fn new(line: &'a str, offset: usize) -> Self {
        Self {
            line,
            offset,
            rev: false,

            allow_skip: true,
        }
    }

    #[must_use]
    pub const fn new_rev(line: &'a str, offset: usize) -> Self {
        Self {
            line,
            offset,
            rev: true,

            allow_skip: true,
        }
    }

    #[must_use]
    pub const fn with_skip(mut self, allow_skip: bool) -> Self {
        self.allow_skip = allow_skip;
        self
    }

    #[must_use]
    pub fn find(self) -> usize {
        match (self.rev, self.offset) {
            (true, 0) => 0,
            (true, _) => self.offset - self.offset_until_target(self.get_rev_it()),
            (false, _) => self.offset + self.offset_until_target(self.get_it()),
        }
    }

    fn offset_until_target(&self, mut chars: Peekable<impl Iterator<Item = char>>) -> usize {
        let mut diff = 0;

        // Part one: skip whitespace if any
        let whitespace_consumed = consume_whitespace(&mut chars);
        diff += whitespace_consumed;

        let Some(mut current_char) = chars.next() else {
            return diff;
        };

        let Some(&next_char) = chars.peek() else {
            return diff;
        };

        if self.allow_skip {
            // Part two: hop to the next word if it it current's word end
            let (hopped, new_current_char) =
                Self::hop_to_next_word(&mut chars, next_char, current_char, whitespace_consumed);
            diff += hopped;
            current_char = new_current_char;
        }

        // Part 3: get the current character's group and
        // iterate until some other group is found
        let current_group = CharGroup::new(current_char);
        diff + Self::skip_to_different_group(chars, &current_group)
    }

    fn hop_to_next_word(
        chars: &mut Peekable<impl Iterator<Item = char>>,
        next_char: char,
        mut current_char: char,
        whitespace_consumed: usize,
    ) -> (usize, char) {
        let mut diff = 0;

        let is_at_end = CharGroup::new(next_char).ne(&CharGroup::new(current_char));
        if is_at_end && whitespace_consumed != 0 {
            return (diff, current_char);
        }
        if is_at_end {
            diff += 1;
            current_char = next_char;
            let _ = chars.next();
        }

        if next_char == ' ' {
            diff += consume_whitespace(chars);
            let Some(new_current_char) = chars.next() else {
                return (diff, current_char);
            };
            diff += 1;
            current_char = new_current_char;
        }

        (diff, current_char)
    }

    fn skip_to_different_group(
        chars: Peekable<impl Iterator<Item = char>>,
        current_group: &CharGroup,
    ) -> usize {
        let mut diff = 0;
        for char in chars {
            if CharGroup::new(char).ne(current_group) {
                break;
            }
            diff += 1;
        }

        diff
    }

    fn get_it(&self) -> Peekable<impl Iterator<Item = char> + '_> {
        consume_n(self.line.chars(), self.offset).peekable()
    }

    fn get_rev_it(&self) -> Peekable<impl Iterator<Item = char> + '_> {
        let offset_from_end = self.line.len() - self.offset;
        consume_n(self.line.chars().rev(), offset_from_end.saturating_sub(1)).peekable()
    }
}

/// Returns character offset of the first non-whitespace character in a line
#[must_use]
pub fn character_start(s: &str) -> usize {
    s.chars()
        .enumerate()
        .find_map(|(i, c)| (!c.is_whitespace()).then_some(i))
        .unwrap_or(0)
}

#[derive(Debug, PartialEq, Eq)]
enum CharGroup {
    Space,
    Punct,
    Alphanumeric,
    Other,
}

impl CharGroup {
    fn new(c: char) -> Self {
        match c {
            _ if c.is_whitespace() => Self::Space,
            '[' | ']' | '(' | ')' | '{' | '}' | '.' | ',' | ':' | ';' => Self::Punct,
            _ if c.is_alphanumeric() => Self::Alphanumeric,
            _ => Self::Other,
        }
    }
}

fn consume_n<T, It>(mut it: It, n: usize) -> It
where
    It: Iterator<Item = T>,
{
    if n == 0 {
        return it;
    }
    let _ = it.nth(n - 1);
    it
}

fn consume_whitespace(it: &mut Peekable<impl Iterator<Item = char>>) -> usize {
    let mut count = 0;
    while it.next_if_eq(&' ').is_some() {
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use crate::string::search::Searcher;

    #[test]
    fn current_word_end() {
        let cases = [
            (("hello", 0), 4),
            (("hello ", 0), 4),
            (("hello1", 0), 5),
            (("hello1", 2), 5),
            (("hello 1231", 0), 4),
            (("hello 1231", 4), 9),
            (("hello 1231", 5), 9),
            (("hello) 1231", 0), 4),
            (("hello{ 1231", 5), 10),
            (("hello{foo", 4), 5),
            (("hello{{foo", 4), 6),
            (("hello:}}.,:foo", 4), 10),
        ];

        for ((line, offset), expected) in cases {
            assert_eq!(
                expected,
                Searcher::new(line, offset).find(),
                "{line}, {offset}",
            );
        }
    }

    #[test]
    fn current_word_start() {
        let cases = [
            (("hello", 4), 0),
            (("hello ", 5), 0),
            (("hello1", 0), 0),
            (("hello1", 2), 0),
            (("hello 1231", 7), 6),
            (("hello 1231", 6), 0),
            (("hello) 1231", 5), 0),
            (("hello{ 1231", 6), 5),
            (("hello{foo", 6), 5),
            (("hello{{foo", 7), 5),
            (("hello:}}.,:foo", 10), 5),
        ];

        for ((line, offset), expected) in cases {
            assert_eq!(
                expected,
                Searcher::new_rev(line, offset).find(),
                "{line}, {offset}",
            );
        }
    }
}
