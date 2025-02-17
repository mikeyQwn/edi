//! Draw-related buffer functionality

use crate::{
    log,
    rope::iter::LineInfo,
    string::highlight::{Highlight, Type},
    terminal::{
        escaping::ANSIColor,
        window::{Cell, Window},
    },
    vec2::Vec2,
};

use super::Buffer;

#[derive(Debug)]
pub struct FlushOptions {
    pub wrap: bool,
    pub highlights: Vec<Highlight>,
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
}

impl Default for FlushOptions {
    fn default() -> Self {
        Self {
            wrap: true,
            highlights: Vec::new(),
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
    pub fn flush(&self, window: &mut Window, opts: &FlushOptions) {
        let mut flush_state = FlushState::new(&opts.highlights);

        let lines = self.inner.lines().skip(self.line_offset).take(self.size.y);
        window.clear();
        log::debug!(
            "buffer::flush cursor_offset: {} opts: {:?}",
            self.cursor_offset,
            opts
        );

        lines.for_each(|li| {
            self.flush_line(window, opts, li, &mut flush_state);
        });

        log::debug!("buffer::flush finished");
    }

    fn flush_line(
        &self,
        window: &mut Window,
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

        if draw_pos.y > self.size.y {
            return;
        }

        draw_pos.x = 0;

        if contents.is_empty() && character_offset == self.cursor_offset {
            window.set_cursor(*draw_pos);
            *found_cursor = true;
        }

        for (i, c) in contents.chars().enumerate() {
            if char::is_control(c) {
                unimplemented!("control characters are not supported yet");
            }
            let character_offset = character_offset + i;

            if self.cursor_offset == character_offset {
                window.set_cursor(*draw_pos);
                *found_cursor = true;
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

            let color =
                Self::get_highlight_color(character_offset, highlights).unwrap_or(ANSIColor::White);

            window.put_cell(*draw_pos, Cell::new(c, color));
            draw_pos.x += 1;
        }

        if !*found_cursor && self.cursor_offset == character_offset + length {
            window.set_cursor(*draw_pos);
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
