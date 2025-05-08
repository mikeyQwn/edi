pub mod draw;

use crate::{
    debug,
    rope::{iter::LineInfo, Rope},
    string::{position::LinePosition, search},
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
    pub fn new(inner: String) -> Self {
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
        debug!("buffer::move_cursor offs: {}", self.cursor_offset);
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
                if self.current_line() == 0 || self.inner.lines().count() == 0 {
                    self.cursor_offset = 0;
                    return;
                }

                let offs = self.cursor_offset
                    - self
                        .inner
                        .line_info(self.current_line())
                        .expect("current line should be in the rope")
                        .character_offset;

                self.set_cursor_line(self.current_line().saturating_sub(steps), offs);
            }
            Direction::Down => {
                if self.inner.lines().count() == 0 {
                    return;
                }

                let offs = self.cursor_offset
                    - self
                        .inner
                        .line_info(self.current_line())
                        .expect("current line should be in the rope")
                        .character_offset;

                self.set_cursor_line(
                    (self.current_line() + steps).min(self.inner.lines().count() - 1),
                    offs,
                );
            }
        }
    }

    fn set_cursor_line(&mut self, line: usize, offs: usize) -> bool {
        let Some(line_info) = self.inner.line_info(line) else {
            return false;
        };

        self.cursor_offset = line_info.character_offset + line_info.length.min(offs);

        true
    }

    pub fn move_to(&mut self, position: LinePosition) {
        let current_line = self.current_line();
        let Some(LineInfo {
            character_offset,
            length,
            contents,
            ..
        }) = self.inner.line(current_line)
        else {
            return;
        };

        self.cursor_offset = match position {
            LinePosition::Start => character_offset,
            LinePosition::End => character_offset + length,
            LinePosition::CharacterStart => character_offset + search::character_start(&contents),
        }
    }

    #[must_use]
    pub fn current_line(&self) -> usize {
        let mut curr_line = 0;
        for line_info in self.inner.lines().parse_contents(false) {
            if line_info.character_offset > self.cursor_offset {
                break;
            }
            curr_line = line_info.line_number;
        }
        curr_line
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::vec2::Vec2;

    use super::*;

    fn test_inputs(inner: &str, n: usize) {
        let mut r = Buffer::new(inner.to_string());
        let mut lines: Vec<_> = inner.lines().map(|v| v.to_owned()).collect();
        let mut expected_pos = Vec2::new(0, 0);
        let mut rng = rand::thread_rng();

        for _ in 0..n {
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

            if rng.gen_range(0..16) < 1 {
                r.write('c');
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

            assert_eq!(r.cursor_offset, cursor_offs);
        }
    }

    #[test]
    fn movement() {
        const TRIES: usize = 1024;

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
        let mut b = Buffer::new(String::new());
        b.write('c');
        b.write('c');
    }
}
