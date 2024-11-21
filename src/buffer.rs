use crate::{
    escaping::ANSIColor,
    log,
    vec2::Vec2,
    window::{Cell, Window},
};

/// A range of bytes to highlight with a color
/// The first element is the range [start, end), where start and end are byte offsets
/// The second element is the color to highlight with
type HighlightRange = (Vec2<usize>, ANSIColor);

fn find_color_at_offset(pos: usize, highlights: &[HighlightRange]) -> Option<ANSIColor> {
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
    pub fn with_wrap(mut self, wrap: bool) -> Self {
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

pub struct Buffer {
    pub inner: String,
    pub size: Vec2<usize>,
}

impl Buffer {
    #[must_use]
    pub const fn new(inner: String, size: Vec2<usize>) -> Self {
        Self { inner, size }
    }

    pub fn flush(&self, window: &mut Window, opts: FlushOptions) {
        let iter = LineIter::new(self);
        let mut pos = Vec2::new(0, 0);
        let mut idx = 0;

        for ev in iter {
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

            idx += 1;
        }
    }

    pub fn inner(&self) -> &str {
        &self.inner
    }
}

enum IterEvent {
    Char(char),
    Control(char),
    Newline,
}

struct LineIter<'a>(std::str::Chars<'a>);

impl<'a> LineIter<'a> {
    fn new(buffer: &'a Buffer) -> Self {
        Self(buffer.inner.chars())
    }
}

impl<'a> Iterator for LineIter<'a> {
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
