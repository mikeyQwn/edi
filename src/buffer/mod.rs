pub mod draw;

use crate::{
    debug,
    rope::{iter::LineInfo, Rope},
    string::{
        position::{GlobalPosition, LinePosition},
        search,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Default)]
pub struct Buffer {
    pub inner: Rope,
    pub cursor_offset: usize,
}

impl Buffer {
    #[must_use]
    pub fn new(inner: &str) -> Self {
        Self {
            inner: Rope::from(inner),

            ..Default::default()
        }
    }

    pub fn write(&mut self, c: char) {
        self.inner
            .insert(self.cursor_offset, c.to_string().as_ref());

        if c == '\n' {
            self.cursor_offset += 1;
            return;
        }

        self.cursor_offset += 1;
    }

    pub fn delete(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            self.inner.delete(self.cursor_offset..=self.cursor_offset);
        }
    }

    /// # Panics
    ///
    /// Never panics
    pub fn move_cursor(&mut self, direction: Direction, steps: usize) {
        match direction {
            Direction::Left => {
                let new_offset = self.cursor_offset.saturating_sub(steps);
                if let Some(c) = self.inner.get(new_offset) {
                    if c != '\n' {
                        self.cursor_offset = new_offset;
                    }
                }
            }
            Direction::Right => {
                let Some(prev) = self.inner.get(self.cursor_offset) else {
                    return;
                };

                if prev == '\n' {
                    return;
                }

                let new_offset = self.cursor_offset + steps;
                self.cursor_offset = new_offset;
            }
            Direction::Up => {
                if self.current_line() == 0 || self.inner.total_lines() == 0 {
                    self.cursor_offset = 0;
                    return;
                }

                let current_line = self.current_line();

                let offs = self
                    .cursor_offset
                    .saturating_sub(self.current_line_info().character_offset);

                self.set_cursor_line(current_line.saturating_sub(steps), offs);
            }
            Direction::Down => {
                if self.inner.total_lines() == 0 {
                    debug!("total lines is zero");
                    return;
                }

                let current_line = self.current_line();
                debug!("curr_line: {}", current_line);

                let offs = self
                    .cursor_offset
                    .saturating_sub(self.current_line_info().character_offset);

                self.set_cursor_line(current_line + steps, offs);
            }
        }
    }

    fn current_line_info(&self) -> LineInfo {
        let current_line = self.current_line();
        self.inner
            .lines()
            .nth(current_line)
            .unwrap_or_else(|| LineInfo {
                character_offset: self.inner.len(),
                line_number: current_line,
                length: 0,
                contents: String::new(),
            })
    }

    fn set_cursor_line(&mut self, line: usize, offs: usize) {
        let total_lines = self.inner.total_lines();
        let actual_line = line.min(total_lines);
        debug!(
            "setting cursor to line: {line} (actual {}),  offs: {offs}, total_lines: {}",
            actual_line,
            self.inner.total_lines()
        );
        let Some(line_info) = self
            .inner
            .line_info(actual_line)
            .or_else(|| self.inner.line_info(actual_line.saturating_sub(1)))
        else {
            return;
        };

        self.cursor_offset = line_info.character_offset + line_info.length.min(offs);
    }

    pub fn move_in_line(&mut self, position: LinePosition) {
        let current_line = self.current_line();
        let Some(LineInfo {
            mut character_offset,
            length,
            mut contents,
            ..
        }) = self.inner.line(current_line)
        else {
            return;
        };

        self.cursor_offset = match position {
            LinePosition::Start => character_offset,
            LinePosition::End => character_offset + length,
            LinePosition::CharacterStart => character_offset + search::character_start(&contents),
            LinePosition::CurrentWordEnd => {
                let is_at_eol = self.cursor_offset - character_offset == length.saturating_sub(1);
                let offset = if is_at_eol {
                    0
                } else {
                    self.cursor_offset - character_offset
                };
                if is_at_eol {
                    let Some(next_line) = self.inner.line(current_line + 1) else {
                        // at the end of the file, nothing we can do
                        return;
                    };
                    contents = next_line.contents;
                    character_offset = next_line.character_offset;
                }
                character_offset + search::current_word_end(&contents, offset)
            }
        }
    }

    pub fn move_global(&mut self, position: GlobalPosition) {
        match position {
            GlobalPosition::Start => self.cursor_offset = 0,
            GlobalPosition::End => self.cursor_offset = self.inner.len().saturating_sub(1),
        }
    }

    #[must_use]
    pub fn current_line(&self) -> usize {
        self.inner.line_of_index(self.cursor_offset)
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::SmallRng, Rng, SeedableRng};

    use crate::{log::set_debug, vec2::Vec2};

    use super::*;

    fn test_inputs(inner: &str, n: usize) {
        let mut r = Buffer::new(inner);
        let mut lines: Vec<_> = inner.lines().map(|v| v.to_owned()).collect();
        let mut expected_pos = Vec2::new(0, 0);
        let mut rng = SmallRng::from_seed([1; 32]);

        for _ in 0..n {
            let original_pos = expected_pos;
            let original_rope_pos = r.cursor_offset;
            let dir = rng.gen_range(0..4);
            let dir = match dir {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                3 => Direction::Right,
                _ => unreachable!(),
            };
            r.move_cursor(dir, 1);
            match dir {
                Direction::Up => {
                    if expected_pos.y > 0 {
                        expected_pos.y -= 1;
                        expected_pos.x = expected_pos.x.min(lines[expected_pos.y].len())
                    } else {
                        expected_pos.x = 0;
                    }
                }
                Direction::Down => {
                    if expected_pos.y + 1 < lines.len() {
                        expected_pos.y += 1;
                        expected_pos.x = expected_pos.x.min(lines[expected_pos.y].len())
                    }
                }
                Direction::Left => {
                    if expected_pos.x > 0 {
                        expected_pos.x -= 1;
                    }
                }
                Direction::Right => {
                    if expected_pos.x + 1 <= lines[expected_pos.y].chars().count() {
                        expected_pos.x += 1;
                    }
                }
            }

            let mut moved = false;
            if rng.gen_range(0..16) < 1 {
                r.write('c');
                moved = true;
                let s = format!(
                    "{}{}{}",
                    &lines[expected_pos.y][..expected_pos.x],
                    'c',
                    &lines[expected_pos.y][expected_pos.x..]
                );
                lines[expected_pos.y] = s;
                expected_pos.x += 1;
            }

            let mut cursor_offs: usize = lines
                .iter()
                .take(expected_pos.y)
                .map(|v| v.chars().count() + 1)
                .sum();

            cursor_offs += expected_pos.x;

            assert_eq!(
                r.cursor_offset, cursor_offs,
                "after: {dir:?}, string: {string:?}, moved: {moved}, expected_pos: {expected_pos:?}, lines: {lines:?}, original pos: {original_pos:?}, original buffer pos: {original_rope_pos}, tree:{tree}",
                string = r.inner.chars().collect::<String>(),
                tree = r.inner.to_ascii_tree(),
            );
        }
    }

    #[test]
    fn movement() {
        const TRIES: usize = 1024;
        set_debug(true);
        crate::log::init().unwrap();

        test_inputs("c\n\n", TRIES);
        test_inputs("\n\n", TRIES);
        test_inputs("\nHe", TRIES);
        test_inputs("Lo\nHe", TRIES);
        test_inputs("He\nllo", TRIES);
        test_inputs("\n", TRIES);
        test_inputs("He", TRIES);
        test_inputs("He\n", TRIES);
        test_inputs("He\nllo\n", TRIES);
        test_inputs("He\nllo\n\n", TRIES);
        test_inputs("\nHe\nllo\n\n", TRIES);
    }

    #[test]
    fn empty() {
        let mut b = Buffer::new("");
        b.write('c');
        b.write('c');
    }
}
