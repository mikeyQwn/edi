use crate::{
    escaping::ANSIColor,
    log,
    rope::{iter::Chars, Rope},
    vec2::Vec2,
    window::{Cell, Window},
};

/// A range of bytes to highlight with a color
/// The first element is the range [start, end), where start and end are byte offsets
/// The second element is the color to highlight with
type HighlightRange = (Vec2<usize>, ANSIColor);

const fn find_color_at_offset(pos: usize, highlights: &[HighlightRange]) -> Option<ANSIColor> {
    let (mut l, mut r) = (0, highlights.len());
    while l < r {
        let m = l + (r - l) / 2;
        let (range, color) = &highlights[m];
        if pos >= range.x && pos < range.y {
            return Some(*color);
        } else if pos < range.x {
            r = m;
        } else {
            l = m + 1;
        }
    }

    None
}

#[derive(Debug)]
pub struct FlushOptions {
    wrap: bool,
    highlights: Vec<HighlightRange>,
}

impl FlushOptions {
    #[must_use]
    pub const fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub fn with_highlights(mut self, mut highlights: Vec<(Vec2<usize>, ANSIColor)>) -> Self {
        highlights.sort_by_key(|(p, _)| p.x);
        self.highlights = highlights;
        self
    }
}

impl Default for FlushOptions {
    fn default() -> Self {
        Self {
            wrap: true,
            highlights: Vec::new(),
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
        let iter = LineIter::new(self);
        let mut pos = Vec2::new(0, 0);
        let mut line_nr = 0;

        log::debug!(
            "buffer::flush cursor_offset: {}, cursor_eol: {}",
            self.cursor_offset,
            self.cursor_eol
        );

        window.clear();
        for (idx, ev) in iter.enumerate() {
            if self.cursor_eol && idx == self.cursor_offset {
                log::debug!("buffer::flush found eol");
                let new_pos = Vec2::new(pos.x + 1, pos.y);
                window.set_cursor(new_pos);
            } else if !self.cursor_eol && idx == self.cursor_offset {
                log::debug!(
                    "buffer::flush setting cursor to {:?}, cause cursor_offset: {:?}",
                    pos,
                    self.cursor_offset
                );
                window.set_cursor(pos);
            }

            match ev {
                IterEvent::Newline => {
                    line_nr += 1;

                    if self.line_offset >= line_nr {
                        continue;
                    }

                    pos.y += 1;
                    pos.x = 0;
                }

                IterEvent::Control(c) => {
                    log::debug!("buffer::flush control character {:?}", c);
                }

                IterEvent::Char(c) => {
                    if self.line_offset > line_nr {
                        continue;
                    }

                    if opts.wrap && pos.x >= self.size.x {
                        pos.y += 1;
                        pos.x = 0;
                    }

                    if pos.y >= self.size.y {
                        break;
                    }

                    let color =
                        find_color_at_offset(idx, &opts.highlights).unwrap_or(ANSIColor::White);

                    window.put_cell(pos, Cell::new(c, color));
                    pos.x += 1;
                }
            }
        }
    }

    pub fn write(&mut self, c: char) {
        let offs = if self.cursor_eol && self.inner.len() != 0 {
            self.cursor_offset + 1
        } else {
            self.cursor_offset
        };
        log::debug!(
            "buffer::write offs: {offs}, cursor_eol: {}",
            self.cursor_eol
        );
        self.inner.insert(offs, c.to_string().as_ref());
        self.cursor_offset += 1;
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
                let Some(new_offset) = self.inner.prev_line_start(self.cursor_offset) else {
                    log::debug!("buffer::move_cursor new_offset not found");
                    return;
                };

                self.cursor_eol = false;

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

enum IterEvent {
    Char(char),
    Control(char),
    Newline,
}

struct LineIter<'a>(Chars<'a>);

impl<'a> LineIter<'a> {
    fn new(buffer: &'a Buffer) -> Self {
        let chars = buffer.inner.chars();
        Self(chars)
    }
}

impl Iterator for LineIter<'_> {
    type Item = IterEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.0.next()?;

        let ev = match c {
            '\n' => IterEvent::Newline,
            c if c.is_control() => IterEvent::Control(c),
            c => IterEvent::Char(c),
        };

        Some(ev)
    }
}

#[cfg(test)]
mod tests {
    use crate::vec2::Vec2;
    use rand::Rng;

    use super::*;

    fn test_movement(inner: &str, n: usize) {
        let mut r = Buffer::new(inner.to_string(), Vec2::new(10, 10));
        let lines: Vec<_> = inner.lines().collect();
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
            println!("{dir:?}");
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

            assert_eq!(r.cursor_offset, cursor_offs);
            assert_eq!(r.cursor_eol, eol);
        }
    }

    #[test]
    fn movement() {
        const TRIES: usize = 1024 * 8;

        test_movement("\n\n", TRIES);
        test_movement("\nHe", TRIES);
        test_movement("Lo\nHe", TRIES);
        test_movement("He\nllo", TRIES);
        test_movement("\n", TRIES);
        test_movement("He", TRIES);
        test_movement("He\n", TRIES);
        test_movement("He\nllo\n", TRIES);
        test_movement("He\nllo\n\n", TRIES);
        test_movement("\nHe\nllo\n\n", TRIES);
    }
}
