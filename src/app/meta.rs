use std::path::PathBuf;

use crate::buffer::FlushOptions;

#[derive(Default, Debug)]
pub struct BufferMeta {
    pub flush_options: FlushOptions,
    pub filepath: Option<PathBuf>,
}

impl BufferMeta {
    pub fn with_filepath(mut self, filepath: Option<PathBuf>) -> Self {
        self.filepath = filepath;
        self
    }
}
