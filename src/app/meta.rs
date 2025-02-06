use std::path::PathBuf;

use edi::fs::Filetype;

use crate::buffer::FlushOptions;

#[derive(Default, Debug)]
pub struct BufferMeta {
    pub flush_options: FlushOptions,
    pub filepath: Option<PathBuf>,
    pub filetype: Filetype,
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
}
