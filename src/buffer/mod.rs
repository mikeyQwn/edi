pub mod draw;

use crate::{log, rope::Rope, vec2::Vec2};

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
    pub size: Vec2<usize>,
    pub cursor_offset: usize,
    pub line_offset: usize,
    pub current_line: usize,
    pub line_count: usize,
}

impl Buffer {
    #[must_use]
    pub fn new(inner: String, dimensions: Vec2<usize>) -> Self {
        let line_count = inner.lines().count();

        Self {
            inner: Rope::from(inner),
            size: dimensions,
            line_count,

            ..Default::default()
        }
    }

    pub fn write(&mut self, c: char) {
        self.inner
            .insert(self.cursor_offset, c.to_string().as_ref());

        if c == '\n' {
            self.cursor_offset += 1;
            self.current_line += 1;
            self.line_count += 1;
            if self.current_line >= self.size.y + self.line_offset {
                self.line_offset += 1;
            }

            return;
        }

        self.cursor_offset += 1;
    }

    pub fn delete(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            if let Some('\n') = self.inner.get(self.cursor_offset) {
                self.line_count -= 1;
                self.current_line -= 1;
            }
            self.inner.delete(self.cursor_offset..=self.cursor_offset);
        }
    }

    /// # Panics
    ///
    /// Never panics
    pub fn move_cursor(&mut self, direction: Direction, steps: usize) {
        log::debug!("buffer::move_cursor offs: {}", self.cursor_offset);
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
                if self.current_line == 0 || self.line_count == 0 {
                    self.cursor_offset = 0;
                    return;
                }

                let offs = self.cursor_offset
                    - self
                        .inner
                        .line_info(self.current_line)
                        .expect("current line should be in the rope")
                        .character_offset;

                self.set_cursor_line(self.current_line.saturating_sub(steps), offs);
            }
            Direction::Down => {
                if self.line_count == 0 {
                    return;
                }

                let offs = self.cursor_offset
                    - self
                        .inner
                        .line_info(self.current_line)
                        .expect("current line should be in the rope")
                        .character_offset;

                self.set_cursor_line((self.current_line + steps).min(self.line_count - 1), offs);
            }
        }
    }

    fn set_cursor_line(&mut self, line: usize, offs: usize) -> bool {
        let Some(line_info) = self.inner.line_info(line) else {
            return false;
        };

        self.current_line = line;

        if self.current_line < self.line_offset {
            self.line_offset = self.current_line;
        }

        if self.current_line >= self.size.y + self.line_offset {
            self.line_offset = self.current_line - self.size.y + 1;
        }

        self.cursor_offset = line_info.character_offset + line_info.length.min(offs);

        true
    }

    // TODO: properly test '\n' offsets
    pub fn move_to(&mut self, offset: usize) {
        let start = self.cursor_offset.min(offset);
        let end = self.cursor_offset.max(offset);
        let lines = self.inner.substr(start..end).filter(|&c| c == '\n').count();
        if offset == start {
            self.current_line += lines;
        } else {
            self.current_line -= lines;
        }

        if self.current_line < self.line_offset {
            self.line_offset = self.current_line;
        }

        if self.current_line >= self.size.y + self.line_offset {
            self.line_offset = self.current_line - self.size.y + 1;
        }

        self.cursor_offset = offset.min(self.inner.len());
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    fn test_inputs(inner: &str, n: usize) {
        let mut r = Buffer::new(inner.to_string(), Vec2::new(10, 10));
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
            assert_eq!(r.current_line, expected_pos.y);
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
        let mut b = Buffer::new(String::new(), Vec2::new(10, 10));
        b.write('c');
        b.write('c');
    }
}
