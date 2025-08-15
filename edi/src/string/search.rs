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

// TODO: this is a mess, refactor to state machine maybe>

/// Returns character offset of the end of the current word OR next word
/// if the search head is already at offset
#[must_use]
pub fn current_word_end(line: &str, offset: usize) -> usize {
    let mut pos = offset;

    let mut chars = consume_n(line.chars(), offset).peekable();

    // Part one: skip whitespace if any
    pos += consume_whitespace(&mut chars);

    // Part two: hop to the next word if it it current's word end
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

    // Part 3: get the current character's group and
    // iterate until some other group is found
    let current_group = CharGroup::new(current_char);

    for char in chars {
        if CharGroup::new(char).ne(&current_group) {
            break;
        }
        pos += 1;
    }

    pos
}

#[must_use]
pub fn current_word_start(line: &str, offset: usize) -> usize {
    if line.len() == 0 || offset == 0 {
        return 0;
    }

    let mut pos = offset;

    let offset_from_end = line.len() - offset;
    let mut chars = consume_n(line.chars().rev(), offset_from_end.saturating_sub(1)).peekable();

    // Part one: skip whitespace if any
    let whitespace_consumed = consume_whitespace(&mut chars);
    pos -= whitespace_consumed;

    // Part two: hop to the next word if it it current's word end
    let Some(mut current_char) = chars.next() else {
        return pos;
    };

    let Some(next_char) = chars.peek() else {
        return pos;
    };

    let next_char = *next_char;

    let is_at_start = CharGroup::new(next_char).ne(&CharGroup::new(current_char));
    if is_at_start && whitespace_consumed != 0 {
        return pos;
    }
    if is_at_start {
        pos -= 1;
        current_char = next_char;
        let _ = chars.next();
    }

    if next_char == ' ' {
        pos -= consume_whitespace(&mut chars);
        let Some(new_current_char) = chars.next() else {
            return pos;
        };
        pos -= 1;
        current_char = new_current_char;
    }

    // Part 3: get the current character's group and
    // iterate until some other group is found
    let current_group = CharGroup::new(current_char);

    for char in chars {
        // eprintln!("I hit this {char}");
        if CharGroup::new(char).ne(&current_group) {
            break;
        }
        pos -= 1;
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
                super::current_word_end(line, offset),
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
                super::current_word_start(line, offset),
                "{line}, {offset}",
            );
        }
    }
}
