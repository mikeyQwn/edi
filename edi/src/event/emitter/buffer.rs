use edi::{
    buffer::{self, Direction},
    string::position::{GlobalPosition, LinePosition},
};
use edi_lib::brand::Id;

use crate::{app::state::State, controller::Handle, event::Payload};

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

pub struct Buffer<'a, 'b> {
    id: Id,
    inner: &'a mut buffer::Buffer,
    ctrl: &'b mut Handle<State>,
}

impl<'a, 'b> Buffer<'a, 'b> {
    pub const fn new(id: Id, buf: &'a mut buffer::Buffer, ctrl: &'b mut Handle<State>) -> Self {
        Self {
            id,
            inner: buf,
            ctrl,
        }
    }

    pub fn write(&mut self, c: char) {
        let write_event = Payload::CharWritten {
            buffer_id: self.id,
            offset: self.inner.cursor_offset,
            c,
        };
        self.inner.write(c);
        self.ctrl.add_event(write_event);
    }

    pub fn delete(&mut self) {
        let buffer_id = self.id;
        let offset = self.inner.cursor_offset;
        let Some(deleted_char) = self.inner.delete() else {
            return;
        };
        let delete_event = Payload::CharDeleted {
            buffer_id,
            offset,
            c: deleted_char,
        };
        self.ctrl.add_event(delete_event);
    }

    pub const fn set_cursor_offset(&mut self, cursor_offset: usize) {
        self.inner.cursor_offset = cursor_offset;
    }

    proxy_method!(fn move_cursor(&mut self, direction: Direction, steps: usize));
    proxy_method!(fn move_global(&mut self, position: GlobalPosition));
    proxy_method!(fn move_in_line(&mut self, position: LinePosition));

    pub const fn ctrl(&mut self) -> &mut Handle<State> {
        self.ctrl
    }
}

impl AsRef<buffer::Buffer> for Buffer<'_, '_> {
    fn as_ref(&self) -> &buffer::Buffer {
        self.inner
    }
}
