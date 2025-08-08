use edi::buffer;

use crate::event::{emitter, sender::EventBuffer};

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

    pub fn as_split_mut<'a, 'b>(
        &'a mut self,
        event_buffer: &'b mut EventBuffer,
    ) -> (emitter::buffer::Buffer<'a, 'b>, &'a mut meta::BufferMeta) {
        (
            emitter::buffer::Buffer::new(&mut self.buffer, event_buffer),
            &mut self.meta,
        )
    }

    pub fn buffer(&self) -> &buffer::Buffer {
        &self.buffer
    }

    #[allow(unused)]
    pub fn buffer_mut<'a, 'b>(
        &'a mut self,
        event_buffer: &'b mut EventBuffer,
    ) -> emitter::buffer::Buffer<'a, 'b> {
        emitter::buffer::Buffer::new(&mut self.buffer, event_buffer)
    }
}
