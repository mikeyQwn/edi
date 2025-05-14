use std::iter::Peekable;

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

/// Returns character offset of the end of the current word OR next word
/// if the search head is already at offset
#[must_use]
pub fn current_word_end(line: &str, offset: usize) -> usize {
    let mut pos = offset;
    let mut chars = line.chars().skip(offset).peekable();

    pos += consume_whitespace(&mut chars);
    let Some(mut current_char) = chars.next() else {
        return pos;
    };

    let Some(next_char) = chars.peek() else {
        return pos;
    };

    let next_char = *next_char;

    let is_at_end = CharGroup::new(next_char).ne(&CharGroup::new(current_char));
    if is_at_end {
        pos += 1;
        current_char = next_char;
        let _ = chars.next();
    }

    if next_char == ' ' {
        pos += consume_whitespace(&mut chars);
        let Some(new_current_char) = chars.next() else {
            return pos;
        };
        pos += 1;
        current_char = new_current_char;
    }

    let current_group = CharGroup::new(current_char);

    for char in chars {
        if CharGroup::new(char).ne(&current_group) {
            break;
        }
        pos += 1;
    }

    pos
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
    #[test]
    fn find_current_word_end() {
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
                super::current_word_end(line, offset),
                expected,
                "{line}, {offset}",
            );
        }
    }
}
