use std::path::PathBuf;

use edi::{
    buffer::{draw::FlushOptions, Buffer},
    fs::Filetype,
    vec2::Vec2,
};

#[derive(Default, Debug)]
pub struct BufferMeta {
    pub flush_options: FlushOptions,
    pub filepath: Option<PathBuf>,
    pub filetype: Filetype,
    pub size: Vec2<usize>,
}

impl BufferMeta {
    pub fn with_filepath(mut self, filepath: Option<PathBuf>) -> Self {
        self.filepath = filepath;
        self
    }

    pub fn with_filetype(mut self, filetype: Filetype) -> Self {
        self.filetype = filetype;
        self
    }

    pub const fn with_size(mut self, size: Vec2<usize>) -> Self {
        self.size = size;
        self
    }

    pub fn normalize(&mut self, buf: &Buffer) {
        let current_line = buf.current_line();
        self.flush_options.line_offset = self.flush_options.line_offset.clamp(
            current_line.saturating_sub(self.size.y.saturating_sub(1)),
            current_line,
        );
    }
}
