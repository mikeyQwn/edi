use std::path::PathBuf;

use edi::buffer::{draw::FlushOptions, Buffer};
use edi_frame::unit::Unit;
use edi_lib::{fs::filetype::Filetype, vec2::Vec2};
use edi_term::coord::Dimensions;

use crate::app::Mode;

#[derive(Debug)]
pub struct BufferMeta {
    pub flush_options: FlushOptions,
    pub filepath: Option<PathBuf>,
    pub filetype: Filetype,
    pub size: Vec2<Unit>,
    pub offset: Vec2<Unit>,
    pub mode: Mode,
}

impl BufferMeta {
    #[must_use]
    pub fn new(mode: Mode) -> Self {
        Self {
            flush_options: FlushOptions::default(),
            filepath: None,
            filetype: Filetype::default(),
            size: Vec2::new(Unit::full_width(), Unit::full_height()),
            offset: Vec2::new(Unit::zero(), Unit::zero()),
            mode,
        }
    }

    pub const fn mode(&self) -> Mode {
        self.mode
    }

    pub const fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
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

    pub fn normalize(&mut self, buf: &Buffer, dimensions: Dimensions<usize>) {
        let y = self.size.y.resolve(dimensions);
        let current_line = buf.current_line();
        self.flush_options.line_offset = self.flush_options.line_offset.clamp(
            current_line.saturating_sub(y.saturating_sub(1)),
            current_line,
        );
    }
}
