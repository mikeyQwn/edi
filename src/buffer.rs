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
    pub cursor_eol: bool,
    pub line_offset: usize,
}

impl Buffer {
    #[must_use]
    pub fn new(inner: String, size: Vec2<usize>) -> Self {
        Self {
            line_offset: 0,
            cursor_offset: 0,
            cursor_eol: inner.is_empty(),
            inner: Rope::from(inner),
            size,
        }
    }

    pub fn text(&self) -> String {
        self.inner.chars().collect()
    }

    pub fn flush(&self, window: &mut Window, opts: &FlushOptions) {
        let mut draw_pos = Vec2::new(0, 0);
        let lines = self.inner.lines().skip(self.line_offset).take(self.size.y);
        window.clear();
        log::debug!("buffer::flush opts: {:?}", opts);

        for LineInfo {
            contents,
            character_offset,
            line_number,
        } in lines
        {
            if draw_pos.y > self.size.y {
                break;
            }

            for (i, c) in contents.chars().enumerate() {
                if char::is_control(c) {
                    unimplemented!("control characters are not supported yet");
                }
                let character_offset = character_offset + i;

                if self.cursor_offset == character_offset {
                    let mut pos = draw_pos;
                    if self.cursor_eol {
                        pos.x += 1;
                    }

                    window.set_cursor(pos);
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
            draw_pos.x = 0;
        }
        log::debug!("buffer::flush finished");
    }

    pub fn write(&mut self, c: char) {
        let offs = if self.cursor_eol && self.inner.len() != 0 {
            self.cursor_offset + 1
        } else {
            self.cursor_offset
        };
        let should_pad = self
            .inner
            .get(self.cursor_offset)
            .filter(|&v| v != '\n')
            .is_none();

        log::debug!(
            "buffer::write offs: {offs}, cursor_eol: {}",
            self.cursor_eol
        );

        self.inner.insert(offs, c.to_string().as_ref());
        if should_pad {
            self.cursor_eol = true;
        } else {
            self.cursor_offset += 1;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            self.inner.delete(self.cursor_offset..=self.cursor_offset);
        }
    }

    pub fn move_cursor(&mut self, steps: usize, direction: Direction) {
        log::debug!(
            "buffer::move_cursor offs: {}, cursor_eol: {}",
            self.line_offset,
            self.cursor_eol
        );
        match direction {
            Direction::Left => {
                if self.cursor_eol {
                    self.cursor_eol = !self.cursor_eol;
                    return;
                }
                let new_offset = self.cursor_offset.saturating_sub(steps);
                if let Some(c) = self.inner.get(new_offset) {
                    if c == '\n' {
                        self.cursor_eol =
                            new_offset != 0 && matches!(self.inner.get(new_offset - 1), Some('\n'));
                    } else {
                        self.cursor_offset = new_offset;
                        self.cursor_eol = false;
                    }
                }
            }
            Direction::Right => {
                let new_offset = self.cursor_offset + steps;
                if self.inner.get(self.cursor_offset) == Some('\n') {
                    return;
                }
                if let Some(c) = self.inner.get(new_offset) {
                    if c == '\n' {
                        self.cursor_eol = !(new_offset == 0
                            || matches!(self.inner.get(new_offset - 1), Some('\n')));
                    } else {
                        self.cursor_offset = new_offset;
                        self.cursor_eol = false;
                    }
                } else {
                    self.cursor_eol =
                        !(new_offset == 0 || matches!(self.inner.get(new_offset - 1), Some('\n')));
                }
            }
            Direction::Up => {
                self.cursor_eol = false;

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
                self.cursor_eol = false;

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

            let eol = lines[expected_pos.y].chars().count() == expected_pos.x
                && !lines[expected_pos.y].is_empty();
            let mut cursor_offs: usize = lines
                .iter()
                .take(expected_pos.y)
                .map(|v| v.chars().count() + 1)
                .sum();

            cursor_offs += expected_pos.x;
            if eol && cursor_offs != 0 {
                cursor_offs -= 1;
            }

            assert_eq!(r.cursor_eol, eol);
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
        println!("{:?}", b);
        b.write('c');
    }
}
