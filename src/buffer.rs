use crate::{
    escaping::ANSIColor,
    log,
    rope::{iter::CharsIter, Rope},
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
}

impl Buffer {
    #[must_use]
    pub fn new(inner: String, size: Vec2<usize>) -> Self {
        Self {
            inner: Rope::from(inner),
            size,
            cursor_offset: 0,
        }
    }

    pub fn text(&self) -> String {
        self.inner.chars().collect()
    }

    pub fn flush(&self, window: &mut Window, opts: &FlushOptions) {
        let iter = LineIter::new(self);
        let mut pos = Vec2::new(0, 0);

        window.clear();
        for (idx, ev) in iter.enumerate() {
            if idx == self.cursor_offset {
                log::debug!(
                    "flush: setting cursor to {:?}, cause cursor_offset: {:?}",
                    pos,
                    self.cursor_offset
                );
                window.set_cursor(pos);
            }

            match ev {
                IterEvent::Newline => {
                    pos.y += 1;
                    pos.x = 0;
                }
                IterEvent::Control(c) => {
                    log::debug!("flush: control character {:?}", c);
                }
                IterEvent::Char(c) => {
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
        self.inner
            .insert(self.cursor_offset, c.to_string().as_ref());
        self.cursor_offset += 1;
    }

    pub fn delete(&mut self) {
        if self.cursor_offset > 0 {
            log::debug!(
                "delete: deleting character at offset {:?}",
                self.cursor_offset
            );
            self.cursor_offset -= 1;
            self.inner.delete(self.cursor_offset..=self.cursor_offset);
        }
    }

    pub fn move_cursor(&mut self, steps: usize, direction: Direction) {
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
                let new_offset = self.cursor_offset + steps;
                if let Some(c) = self.inner.get(new_offset) {
                    if c != '\n' {
                        self.cursor_offset = new_offset;
                    }
                }
            }
            Direction::Up => {
                let Some(new_offset) = self.inner.prev_line_start(self.cursor_offset) else {
                    log::debug!("new_offset not found");
                    return;
                };

                self.cursor_offset = new_offset;
            }
            Direction::Down => {
                let Some(new_offset) = self.inner.next_line_start(self.cursor_offset) else {
                    log::debug!("new_offset not found");
                    return;
                };

                log::debug!("new_offset is {new_offset}");
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

struct LineIter<'a>(CharsIter<'a>);

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
