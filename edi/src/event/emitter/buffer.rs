use edi::{
    buffer::{self, Direction},
    string::position::{GlobalPosition, LinePosition},
};
use edi_lib::brand::Id;

use crate::event::{sender::EventBuffer, Payload};

macro_rules! proxy_method {
    (
        $(#[$meta:meta])*
        fn $name:ident (&mut self $(, $arg:ident : $ty:ty)* ) $(-> $ret:ty)?
    ) => {
        $(#[$meta])*
        pub fn $name(&mut self $(, $arg : $ty)* ) $(-> $ret)? {
            self.inner.$name($($arg),*)
        }
    };
}

#[derive(Debug)]
pub struct Buffer<'a, 'b> {
    id: Id,
    inner: &'a mut buffer::Buffer,
    event_buffer: &'b mut EventBuffer,
}

impl<'a, 'b> Buffer<'a, 'b> {
    pub fn new(id: Id, buf: &'a mut buffer::Buffer, event_buffer: &'b mut EventBuffer) -> Self {
        Self {
            id,
            inner: buf,
            event_buffer,
        }
    }

    pub fn write(&mut self, c: char) {
        let write_event = Payload::CharWritten {
            buffer_id: self.id,
            offset: self.inner.cursor_offset,
            c,
        };
        self.inner.write(c);
        self.event_buffer.add_event(write_event);
    }

    pub fn delete(&mut self) {
        // TODO: Store the deleted character
        let write_event = Payload::CharDeleted {
            buffer_id: self.id,
            offset: self.inner.cursor_offset,
            c: ' ',
        };
        self.inner.delete();
        self.event_buffer.add_event(write_event);
    }

    pub fn set_cursor_offset(&mut self, cursor_offset: usize) {
        self.inner.cursor_offset = cursor_offset
    }

    proxy_method!(fn move_cursor(&mut self, direction: Direction, steps: usize));
    proxy_method!(fn move_global(&mut self, position: GlobalPosition));
    proxy_method!(fn move_in_line(&mut self, position: LinePosition));
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
