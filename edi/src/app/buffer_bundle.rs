use edi::buffer;

use super::meta;

#[derive(Debug)]
pub struct BufferBundle {
    buffer: buffer::Buffer,
    meta: meta::BufferMeta,
}

impl BufferBundle {
    pub fn new(buffer: buffer::Buffer, meta: meta::BufferMeta) -> Self {
        Self { buffer, meta }
    }

    pub fn as_split(&self) -> (&buffer::Buffer, &meta::BufferMeta) {
        (&self.buffer, &self.meta)
    }

    pub fn as_split_mut(&mut self) -> (&mut buffer::Buffer, &mut meta::BufferMeta) {
        (&mut self.buffer, &mut self.meta)
    }

    pub fn buffer(&self) -> &buffer::Buffer {
        &self.buffer
    }

    #[allow(unused)]
    pub fn buffer_mut(&mut self) -> &mut buffer::Buffer {
        &mut self.buffer
    }
}
