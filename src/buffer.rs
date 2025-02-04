use std::collections::HashMap;

use edi::{
    rope::{iter::LineInfo, Rope},
    terminal::{
        escaping::ANSIColor,
        window::{Cell, Window},
    },
    vec2::Vec2,
};

use crate::log;

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
            length,
        } in lines
        {
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

            if !found_cursor && self.cursor_offset == character_offset + length {
                window.set_cursor(draw_pos);
                found_cursor = true;
            }

            draw_pos.y += 1;
        }

        log::debug!("buffer::flush finished");
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
