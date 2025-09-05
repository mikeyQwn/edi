use std::path::PathBuf;

use edi_frame::unit::Unit;
use edi_lib::buffer::{draw::FlushOptions, Buffer};
use edi_lib::{fs::filetype::Filetype, vec2::Vec2};
use edi_term::coord::Dimensions;

use crate::app::Mode;

use super::context::Context;

#[derive(Debug)]
pub struct BufferMeta {
    pub flush_options: FlushOptions,

    pub statusline: bool,
    pub filepath: Option<PathBuf>,
    pub filetype: Filetype,
    pub size: Vec2<Unit>,
    pub offset: Vec2<Unit>,
    pub mode: Mode,

    pub flags: Flags,
}

impl BufferMeta {
    #[must_use]
    pub fn new(mode: Mode) -> Self {
        Self {
            flush_options: FlushOptions::default(),

            statusline: false,
            filepath: None,
            filetype: Filetype::default(),
            size: Vec2::new(Unit::full_width(), Unit::full_height()),
            offset: Vec2::new(Unit::zero(), Unit::zero()),
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

    pub fn updated_flush_options(&mut self, ctx: &Context) -> &mut FlushOptions {
        self.flush_options
            .set_wrap(ctx.settings.word_wrap)
            .set_mode(self.mode.as_str())
            .set_line_numbers(ctx.settings.line_numbers)
            .set_statusline(self.statusline)
    }

    pub fn normalize(&mut self, ctx: &Context, buf: &Buffer, window_dimensions: Dimensions<usize>) {
        let (x, y) = (
            self.size.x.resolve(window_dimensions),
            self.size.y.resolve(window_dimensions),
        );
        let current_line = buf.current_line();
        let opts = self.updated_flush_options(ctx);
        let y = buf
            .main_dimensions(Dimensions::new(x, y), buf.inner.total_lines(), opts)
            .height;

        self.flush_options.line_offset = self.flush_options.line_offset.clamp(
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
