use edi::buffer;

use crate::event::sender::EventBuffer;

pub struct Buffer<'a, 'b> {
    inner: &'a mut buffer::Buffer,
    event_buffer: &'b mut EventBuffer,
}

impl<'a, 'b> Buffer<'a, 'b> {
    pub fn new(buf: &'a mut buffer::Buffer, event_buffer: &'b mut EventBuffer) -> Self {
        Self {
            inner: buf,
            event_buffer,
        }
    }
}

impl<'a, 'b> AsRef<buffer::Buffer> for Buffer<'a, 'b> {
    fn as_ref(&self) -> &buffer::Buffer {
        self.inner
    }
}

// TODO: Remove this when everyting is moved over to methods
impl<'a, 'b> AsMut<buffer::Buffer> for Buffer<'a, 'b> {
    fn as_mut(&mut self) -> &mut buffer::Buffer {
        self.inner
    }
}
