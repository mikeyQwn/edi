//! Draw-related buffer functionality

use crate::{
    draw::Surface,
    log,
    rope::iter::LineInfo,
    string::highlight::{Highlight, Type},
    terminal::{escaping::ANSIColor, window::Cell},
    vec2::Vec2,
};

use super::Buffer;

#[derive(Debug)]
pub struct FlushOptions {
    pub wrap: bool,
    pub highlights: Vec<Highlight>,
    pub line_offset: usize,
}

impl FlushOptions {
    #[must_use]
    pub const fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub fn with_highlights(mut self, highlights: Vec<Highlight>) -> Self {
        self.highlights = highlights;
        self
    }

    #[must_use]
    pub const fn with_line_offset(mut self, line_offset: usize) -> Self {
        self.line_offset = line_offset;
        self
    }
}

impl Default for FlushOptions {
    fn default() -> Self {
        Self {
            wrap: true,
            highlights: Vec::new(),
            line_offset: 0,
        }
    }
}

struct FlushState<'a> {
    draw_pos: Vec2<usize>,
    found_cursor: bool,
    highlights: &'a [Highlight],
}

impl<'a> FlushState<'a> {
    #[must_use]
    pub const fn new(highlights: &'a [Highlight]) -> Self {
        Self {
            draw_pos: Vec2::new(0, 0),
            found_cursor: false,
            highlights,
        }
    }
}

impl Buffer {
    pub fn flush<S: Surface>(&self, surface: &mut S, opts: &FlushOptions) {
        let mut flush_state = FlushState::new(&opts.highlights);

        let lines = self
            .inner
            .lines()
            .skip(opts.line_offset)
            .take(surface.dimensions().y);
        surface.clear();
        log::debug!(
            "buffer::flush cursor_offset: {} opts: {:?}",
            self.cursor_offset,
            opts
        );

        lines.for_each(|li| {
            self.flush_line(surface, opts, li, &mut flush_state);
        });

        log::debug!("buffer::flush finished");
    }

    fn flush_line<S: Surface>(
        &self,
        surface: &mut S,
        opts: &FlushOptions,
        info: LineInfo,
        flush_state: &mut FlushState,
    ) {
        let LineInfo {
            contents,
            character_offset,
            length,
            ..
        } = info;

        let FlushState {
            draw_pos,
            found_cursor,
            highlights,
        } = flush_state;

        if draw_pos.y >= surface.dimensions().y {
            return;
        }

        draw_pos.x = 0;

        if contents.is_empty() && character_offset == self.cursor_offset {
            surface.move_cursor(*draw_pos);
            *found_cursor = true;
        }

        for (i, c) in contents.chars().enumerate() {
            if char::is_control(c) {
                unimplemented!("control characters are not supported yet");
            }
            let character_offset = character_offset + i;

            if self.cursor_offset == character_offset {
                surface.move_cursor(*draw_pos);
                *found_cursor = true;
            }

            match (draw_pos.x >= surface.dimensions().x, opts.wrap) {
                (true, true) => {
                    draw_pos.x = 0;
                    draw_pos.y += 1;
                }
                (true, false) => {
                    break;
                }
                _ => {}
            }

            let color =
                Self::get_highlight_color(character_offset, highlights).unwrap_or(ANSIColor::White);

            surface.set(*draw_pos, Cell::new(c, color).into());
            draw_pos.x += 1;
        }

        if !*found_cursor && self.cursor_offset == character_offset + length {
            surface.move_cursor(*draw_pos);
            *found_cursor = true;
        }

        draw_pos.y += 1;
    }

    fn get_highlight_color(offs: usize, highlights: &mut &[Highlight]) -> Option<ANSIColor> {
        let first_hl = highlights.first()?;

        if first_hl.start + first_hl.len < offs {
            *highlights = &highlights[1..];
            return Self::get_highlight_color(offs, highlights);
        }

        if !(first_hl.start..first_hl.start + first_hl.len).contains(&offs) {
            return None;
        }

        Some(match first_hl.ty {
            Type::Keyword => ANSIColor::Magenta,
            _ => ANSIColor::Red,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        buffer::{draw::FlushOptions, Buffer},
        draw::Surface,
        vec2::Vec2,
    };

    struct TestSurface {
        chars: Vec<Vec<char>>,
        cursor_pos: Option<Vec2<usize>>,
    }

    impl TestSurface {
        pub fn new(dims: Vec2<usize>) -> Self {
            Self {
                chars: vec![vec![' '; dims.x]; dims.y],
                cursor_pos: None,
            }
        }
        pub fn get_contents(&self) -> Vec<String> {
            self.chars.iter().map(|row| row.iter().collect()).collect()
        }
    }

    impl Surface for TestSurface {
        fn set(&mut self, position: Vec2<usize>, cell: crate::draw::Cell) {
            let Vec2 { x, y } = position;
            if y < self.chars.len() && x < self.chars[y].len() {
                self.chars[y][x] = cell.char;
            }
        }
        fn clear(&mut self) {
            let Vec2 { x, y } = self.dimensions();
            self.chars = vec![vec![' '; x]; y];
            self.cursor_pos = None;
        }
        fn dimensions(&self) -> Vec2<usize> {
            Vec2::new(self.chars[0].len(), self.chars.len())
        }
        fn move_cursor(&mut self, point: Vec2<usize>) {
            self.cursor_pos = Some(point)
        }
    }

    #[test]
    fn simple() {
        let mut buf = Buffer::new(String::from("Hello!\nWorld!"));
        buf.cursor_offset = 1;

        let mut surface = TestSurface::new(Vec2::new(10, 5));

        buf.flush(&mut surface, &Default::default());

        let contents = surface.get_contents();
        assert_eq!(contents[0], "Hello!    ");
        assert_eq!(contents[1], "World!    ");

        assert_eq!(surface.cursor_pos, Some(Vec2::new(1, 0)));

        for line in &contents[2..] {
            assert_eq!(line, "          ");
        }
    }

    #[test]
    fn wrap() {
        let long_line = "This is a very long line that should wrap around";
        let mut buf = Buffer::new(String::from(long_line));
        buf.cursor_offset = 11;

        let mut surface = TestSurface::new(Vec2::new(10, 5));

        buf.flush(&mut surface, &Default::default());

        let contents = surface.get_contents();
        assert_eq!(contents[0], "This is a ");
        assert_eq!(contents[1], "very long ");
        assert_eq!(contents[2], "line that ");
        assert_eq!(contents[3], "should wra");
        assert_eq!(contents[4], "p around  ");

        assert_eq!(surface.cursor_pos, Some(Vec2::new(1, 1)));

        let mut surface = TestSurface::new(Vec2::new(10, 5));
        let opts = super::FlushOptions::default().with_wrap(false);
        buf.flush(&mut surface, &opts);

        let contents = surface.get_contents();
        assert_eq!(contents[0], "This is a ");
        for line in &contents[1..] {
            assert_eq!(line, "          ");
        }

        assert_eq!(surface.cursor_pos, None);

        let exact_width = "Exactly10c";
        let buf = Buffer::new(String::from(exact_width));
        let mut surface = TestSurface::new(Vec2::new(10, 2));

        buf.flush(&mut surface, &Default::default());

        let contents = surface.get_contents();
        assert_eq!(contents[0], "Exactly10c");
        assert_eq!(contents[1], "          ");
        assert_eq!(surface.cursor_pos, Some(Vec2::new(0, 0)));

        let with_empty = "First\nVery very long line that wraps\nLast";
        let buf = Buffer::new(String::from(with_empty));
        let mut surface = TestSurface::new(Vec2::new(10, 6));

        buf.flush(&mut surface, &Default::default());

        let contents = surface.get_contents();
        assert_eq!(contents[0], "First     ");
        assert_eq!(contents[1], "Very very ");
        assert_eq!(contents[2], "long line ");
        assert_eq!(contents[3], "that wraps");
        assert_eq!(contents[4], "Last      ");
        assert_eq!(contents[5], "          ");
        assert_eq!(surface.cursor_pos, Some(Vec2::new(0, 0)));
    }

    #[test]
    fn line_offset() {
        let text = "First line\nSecond line\nThird line\nFourth line";
        let buf = Buffer::new(String::from(text));
        let mut surface = TestSurface::new(Vec2::new(20, 3));

        buf.flush(&mut surface, &Default::default());
        let contents = surface.get_contents();
        assert_eq!(contents[0], "First line          ");
        assert_eq!(contents[1], "Second line         ");
        assert_eq!(contents[2], "Third line          ");

        let opts = FlushOptions::default().with_line_offset(1);
        buf.flush(&mut surface, &opts);
        let contents = surface.get_contents();
        assert_eq!(contents[0], "Second line         ");
        assert_eq!(contents[1], "Third line          ");
        assert_eq!(contents[2], "Fourth line         ");

        let opts = FlushOptions::default().with_line_offset(10);
        buf.flush(&mut surface, &opts);
        let contents = surface.get_contents();
        assert_eq!(contents[0], "                    ");
        assert_eq!(contents[1], "                    ");
        assert_eq!(contents[2], "                    ");
    }
}
