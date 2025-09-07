use std::path::PathBuf;

use edi_frame::unit::Unit;
use edi_lib::buffer::{draw::FlushOptions, Buffer};
use edi_lib::string::highlight::Highlight;
use edi_lib::{fs::filetype::Filetype, vec2::Vec2};
use edi_term::coord::UDims;

use crate::app::Mode;

use super::context::Context;

#[derive(Debug)]
pub struct BufferMeta {
    pub statusline: bool,
    pub filepath: Option<PathBuf>,
    pub filetype: Filetype,
    pub size: Vec2<Unit>,
    pub offset: Vec2<Unit>,
    pub line_offset: usize,
    pub highlights: Vec<Highlight>,
    pub line_numbers: bool,

    pub mode: Mode,

    pub flags: Flags,
}

impl BufferMeta {
    #[must_use]
    pub fn new(mode: Mode) -> Self {
        Self {
            statusline: false,
            filepath: None,
            filetype: Filetype::default(),
            size: Vec2::new(Unit::full_width(), Unit::full_height()),
            offset: Vec2::new(Unit::zero(), Unit::zero()),
            line_offset: 0,
            highlights: Vec::new(),
            line_numbers: false,

            mode,

            flags: Flags::empty(),
        }
    }

    pub const fn mode(&self) -> Mode {
        self.mode
    }

    pub const fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn with_statusline(mut self, statusline: bool) -> Self {
        self.statusline = statusline;
        self
    }

    pub fn with_filepath(mut self, filepath: Option<PathBuf>) -> Self {
        self.filepath = filepath;
        self
    }

    pub fn with_filetype(mut self, filetype: Filetype) -> Self {
        self.filetype = filetype;
        self
    }

    pub const fn with_size(mut self, size: Vec2<Unit>) -> Self {
        self.size = size;
        self
    }

    pub const fn with_offset(mut self, offset: Vec2<Unit>) -> Self {
        self.offset = offset;
        self
    }

    pub const fn with_flags(mut self, flags: Flags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_highlights(mut self, highlights: Vec<Highlight>) -> Self {
        self.highlights = highlights;
        self
    }

    pub fn with_line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }

    pub fn set_highlights(&mut self, highlights: Vec<Highlight>) -> &mut Self {
        self.highlights = highlights;
        self
    }

    pub fn updated_flush_options(&mut self, ctx: &Context) -> FlushOptions {
        FlushOptions::default()
            .with_wrap(ctx.settings.word_wrap)
            .with_mode(self.mode.as_str())
            .with_line_numbers(ctx.settings.line_numbers)
            .with_statusline(self.statusline)
            .with_line_offset(self.line_offset)
            .with_highlights(&self.highlights)
    }

    pub fn size_resolved(&self, window_dimensions: UDims) -> Vec2<usize> {
        self.size.map(|coord| coord.resolve(window_dimensions))
    }

    pub fn normalize(&mut self, ctx: &Context, buffer: &Buffer, window_dimensions: UDims) {
        let size_resolved = self.size_resolved(window_dimensions).into_dims();
        let current_line = buffer.current_line();
        let total_lines = buffer.total_lines();
        let opts = self.updated_flush_options(ctx);
        let y = buffer
            .main_dimensions(size_resolved, total_lines, &opts)
            .height;

        self.line_offset = self.line_offset.clamp(
            current_line.saturating_sub(y.saturating_sub(1)),
            current_line,
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Flags(u16);

impl Flags {
    const IS_TERMINAL: u8 = 0;

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn set_is_terminal(self) -> Self {
        self.set(Self::IS_TERMINAL)
    }

    pub fn is_terminal(&self) -> bool {
        self.get(Self::IS_TERMINAL)
    }

    fn set(&self, offs: u8) -> Self {
        Self(self.0 | (1 << offs))
    }

    fn get(&self, offs: u8) -> bool {
        (self.0 & (1 << offs)) != 0
    }
}
