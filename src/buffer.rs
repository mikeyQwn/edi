use std::collections::HashMap;

use crate::{
    escaping::ANSIColor,
    log,
    rope::{iter::LineInfo, Rope},
    vec2::Vec2,
    window::{Cell, Window},
};

/// A range of bytes to highlight with a color
/// The first element is the range [start, end), where start and end are byte offsets
/// The second element is the color to highlight with
pub type Highlight = (Vec2<usize>, ANSIColor);

#[derive(Debug)]
pub struct FlushOptions {
    pub wrap: bool,
    pub highlights: HashMap<usize, Vec<Highlight>>,
}

impl FlushOptions {
    #[must_use]
    pub const fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub fn with_highlights(mut self, highlights: HashMap<usize, Vec<Highlight>>) -> Self {
        self.highlights = highlights;
        self
    }
}

impl Default for FlushOptions {
    fn default() -> Self {
        Self {
            wrap: true,
            highlights: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Buffer {
    pub inner: Rope,
    pub size: Vec2<usize>,
    pub cursor_offset: usize,
    pub line_offset: usize,
}

impl Buffer {
    #[must_use]
    pub fn new(inner: String, size: Vec2<usize>) -> Self {
        Self {
            line_offset: 0,
            cursor_offset: 0,
            inner: Rope::from(inner),
            size,
        }
    }

    pub fn text(&self) -> String {
        self.inner.chars().collect()
    }

    pub fn flush(&self, window: &mut Window, opts: &FlushOptions) {
        let mut draw_pos = Vec2::new(0, 0);
        let mut found_cursor = false;
        let lines = self.inner.lines().skip(self.line_offset).take(self.size.y);
        window.clear();
        log::debug!(
            "buffer::flush cursor_offset: {} opts: {:?}",
            self.cursor_offset,
            opts
        );

        for LineInfo {
            contents,
            character_offset,
            line_number,
        } in lines
        {
            if character_offset > 0 && self.cursor_offset == character_offset - 1 {
                let mut pos = draw_pos;
                pos.y -= 1;
                found_cursor = true;
                window.set_cursor(pos);
            }
            draw_pos.x = 0;

            if draw_pos.y > self.size.y {
                break;
            }

            if contents.is_empty() && character_offset == self.cursor_offset {
                window.set_cursor(draw_pos);
                found_cursor = true;
            }

            for (i, c) in contents.chars().enumerate() {
                if char::is_control(c) {
                    unimplemented!("control characters are not supported yet");
                }
                let character_offset = character_offset + i;

                if self.cursor_offset == character_offset {
                    window.set_cursor(draw_pos);
                    found_cursor = true;
                }

                match (draw_pos.x > self.size.x, opts.wrap) {
                    (true, true) => {
                        draw_pos.x = 0;
                        draw_pos.y += 1;
                    }
                    (true, false) => {
                        break;
                    }
                    _ => {}
                }

                let color = opts
                    .highlights
                    .get(&line_number)
                    .and_then(|hls| {
                        hls.iter()
                            .find(|&&(range, _)| (range.x..range.y).contains(&i))
                    })
                    .map_or(ANSIColor::White, |&(_, col)| col);

                window.put_cell(draw_pos, Cell::new(c, color));
                draw_pos.x += 1;
            }

            draw_pos.y += 1;
        }

        if !found_cursor {
            let mut pos = draw_pos;
            pos.y -= 1;
            window.set_cursor(pos);
        }

        log::debug!("buffer::flush finished");
    }

    pub fn write(&mut self, c: char) {
        let prev = self.inner.get(self.cursor_offset);

        self.inner
            .insert(self.cursor_offset, c.to_string().as_ref());

        if c == '\n' {
            self.cursor_offset += 1;

            let Some((next_line_nr, _)) = self
                .inner
                .line_starts()
                .enumerate()
                .find(|&(_, v)| v == self.cursor_offset)
            else {
                return;
            };

            let next_line_nr = next_line_nr + 1;

            if next_line_nr > self.size.y + self.line_offset {
                self.line_offset += 1;
            }

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

    pub fn move_cursor(&mut self, steps: usize, direction: Direction) {
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
                let Some(new_offset) = self.inner.prev_line_start(self.cursor_offset) else {
                    log::debug!("buffer::move_cursor new_offset not found");
                    return;
                };

                let (line_nr, _) = self
                    .inner
                    .line_starts()
                    .enumerate()
                    .find(|&(_, v)| v == new_offset)
                    .unwrap();

                if line_nr < self.line_offset {
                    self.line_offset -= 1;
                }

                self.cursor_offset = new_offset;
            }
            Direction::Down => {
                let Some(new_offset) = self.inner.next_line_start(self.cursor_offset) else {
                    log::debug!("buffer::move_cursor not found");
                    return;
                };

                let Some((next_line_nr, _)) = self
                    .inner
                    .line_starts()
                    .enumerate()
                    .find(|&(_, v)| v == new_offset)
                else {
                    return;
                };

                let next_line_nr = next_line_nr + 1;

                if next_line_nr > self.size.y + self.line_offset {
                    self.line_offset += 1;
                }

                self.cursor_offset = new_offset;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vec2::Vec2;
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
            r.move_cursor(1, dir);
            match dir {
                Direction::Up => {
                    if expected_pos.y > 0 {
                        expected_pos.y -= 1;
                    }
                    expected_pos.x = 0;
                }
                Direction::Down => {
                    if expected_pos.y + 1 < lines.len() {
                        expected_pos.y += 1;
                        expected_pos.x = 0;
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
        let mut b = Buffer::new(String::new(), Vec2::new(10, 10));
        b.write('c');
        b.write('c');
    }
}
