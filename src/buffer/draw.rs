//! Draw-related buffer functionality

use crate::{
    debug,
    draw::{BoundExt, Cell, Color, Surface},
    rect::Rect,
    rope::iter::LineInfo,
    span,
    string::highlight::{Highlight, Type},
    vec2::Vec2,
};

use super::Buffer;

#[derive(Debug)]
pub struct FlushOptions {
    pub wrap: bool,
    pub line_numbers: bool,
    pub highlights: Vec<Highlight>,
    pub line_offset: usize,
}

#[derive(Debug)]
struct DrawBounds {
    line_numbers: Rect,
    main: Rect,
}

impl FlushOptions {
    #[must_use]
    pub const fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub const fn with_line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
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
            line_numbers: false,
            highlights: Vec::new(),
            line_offset: 0,
        }
    }
}

struct FlushState<'a> {
    current_y: usize,
    highlights: &'a [Highlight],
    bounds: DrawBounds,
}

impl<'a> FlushState<'a> {
    #[must_use]
    pub const fn new(highlights: &'a [Highlight], bounds: DrawBounds) -> Self {
        Self {
            current_y: 0,
            highlights,
            bounds,
        }
    }
}

impl Buffer {
    pub fn flush<S: Surface>(&self, surface: &mut S, opts: &FlushOptions) {
        let _span = span!("buffer::flush");

        let line_number_offset = if opts.line_numbers {
            let total_lines = self.inner.total_lines();
            total_lines.to_string().len() + 1
        } else {
            0
        };
        let Vec2 { x, y } = surface.dimensions();
        let (line_numbers, main) = Rect::new_in_origin(x, y).split_horizontal(line_number_offset);
        let bounds = DrawBounds { line_numbers, main };

        let mut flush_state = FlushState::new(&opts.highlights, bounds);
        // debug!("cursor_offset: {} opts: {:?}", self.cursor_offset, opts);

        self.flush_lines(surface, opts, &mut flush_state);

        debug!("finished");
    }

    fn flush_lines<S: Surface>(
        &self,
        surface: &mut S,
        opts: &FlushOptions,
        state: &mut FlushState,
    ) {
        let available_height = surface.dimensions().y;
        let _span = span!("flush_lines");
        self.inner
            .lines()
            .skip(opts.line_offset)
            .take(available_height)
            .for_each(|line_info| {
                self.flush_line(surface, opts, &line_info, state);
            });
    }

    fn flush_line<S: Surface>(
        &self,
        surface: &mut S,
        opts: &FlushOptions,
        info: &LineInfo,
        flush_state: &mut FlushState,
    ) {
        if flush_state.current_y >= surface.dimensions().y {
            return;
        }

        let mut max_y = flush_state.current_y;

        if opts.line_numbers {
            Self::flush_line_number(info.line_number, flush_state, surface);
        }

        self.flush_main(info, &mut max_y, flush_state, opts, surface);

        flush_state.current_y = max_y + 1;
    }

    fn flush_line_number<S: Surface>(
        line_number: usize,
        flush_state: &FlushState,
        surface: &mut S,
    ) {
        let line_number_str = line_number.to_string();
        let offs = flush_state
            .bounds
            .line_numbers
            .width()
            .saturating_sub(line_number_str.len())
            .saturating_sub(1);

        line_number_str
            .chars()
            .take(flush_state.bounds.line_numbers.width().saturating_sub(1))
            .enumerate()
            .for_each(|(i, c)| {
                flush_state.bounds.line_numbers.set(
                    Vec2::new(offs + i, flush_state.current_y),
                    Cell::new(c, Color::Cyan, Color::None),
                    surface,
                );
            });
    }

    fn flush_main<S: Surface>(
        &self,
        info: &LineInfo,
        max_y: &mut usize,
        flush_state: &mut FlushState,
        opts: &FlushOptions,
        surface: &mut S,
    ) {
        let LineInfo {
            contents: line_contents,
            character_offset: line_character_offset,
            length,
            ..
        } = info;

        let mut x_offset = 0;

        for (idx, character) in line_contents.chars().enumerate() {
            if char::is_control(character) && character != '\t' {
                todo!("control characters are not supported yet");
            }

            let character_offset = line_character_offset + idx;

            let Some(char_pos) = Self::get_char_pos(surface, x_offset, opts, flush_state) else {
                continue;
            };

            *max_y = char_pos.y.max(*max_y);

            x_offset += Self::char_len(character);

            if self.cursor_offset == character_offset {
                flush_state.bounds.main.move_cursor(char_pos, surface);
            }

            let color = Self::get_highlight_color(character_offset, &mut flush_state.highlights)
                .unwrap_or(Color::White);

            match character {
                '\t' => {
                    for i in 0..4 {
                        let new_pos = Vec2::new(char_pos.x + i, char_pos.y);
                        flush_state.bounds.main.set(
                            new_pos,
                            Cell::new(character, color, Color::None),
                            surface,
                        );
                    }
                }
                _ => {
                    flush_state.bounds.main.set(
                        char_pos,
                        Cell::new(character, color, Color::None),
                        surface,
                    );
                }
            }
        }

        if self.cursor_offset == line_character_offset + length {
            if let Some(char_pos) = Self::get_char_pos(surface, x_offset, opts, flush_state) {
                flush_state.bounds.main.move_cursor(char_pos, surface);
            };
        }
    }

    const fn char_len(c: char) -> usize {
        match c {
            '\t' => 4,
            _other => 1,
        }
    }

    fn get_char_pos<S: Surface>(
        surface: &S,
        x_offset: usize,
        opts: &FlushOptions,
        state: &FlushState,
    ) -> Option<Vec2<usize>> {
        let Vec2 { x: w, y: h } = surface.dimensions();
        let y_offset = state.current_y;
        let pos = if opts.wrap {
            Vec2::new(x_offset % w, y_offset + x_offset / w)
        } else {
            Vec2::new(x_offset, y_offset)
        };

        Rect::new_in_origin(w, h).contains_point(pos).then_some(pos)
    }

    fn get_highlight_color(offs: usize, highlights: &mut &[Highlight]) -> Option<Color> {
        let first_hl = highlights.first()?;

        if first_hl.start + first_hl.len < offs {
            *highlights = &highlights[1..];
            return Self::get_highlight_color(offs, highlights);
        }

        if !(first_hl.start..first_hl.start + first_hl.len).contains(&offs) {
            return None;
        }

        Some(match first_hl.ty {
            Type::Keyword => Color::Magenta,
            _ => Color::Red,
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

        pub fn clear(&mut self) {
            self.chars = vec![vec![' '; self.chars[0].len()]; self.chars.len()];
            self.cursor_pos = None;
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
        surface.clear();
        buf.flush(&mut surface, &opts);
        let contents = surface.get_contents();
        assert_eq!(contents[0], "Second line         ");
        assert_eq!(contents[1], "Third line          ");
        assert_eq!(contents[2], "Fourth line         ");

        let opts = FlushOptions::default().with_line_offset(10);
        buf.flush(&mut surface, &opts);
        surface.clear();
        let contents = surface.get_contents();
        assert_eq!(contents[0], "                    ");
        assert_eq!(contents[1], "                    ");
        assert_eq!(contents[2], "                    ");
    }
}
