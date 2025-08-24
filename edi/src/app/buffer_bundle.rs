use edi::buffer;
use edi_lib::brand::Id;

use crate::{
    controller::Handle,
    event::{emitter, sender::EventBuffer},
};

use super::{meta, state::State};

#[derive(Debug)]
pub struct BufferBundle {
    id: Id,
    pub(super) position: usize,
    buffer: buffer::Buffer,
    meta: meta::BufferMeta,
}

impl BufferBundle {
    pub fn new(id: Id, position: usize, buffer: buffer::Buffer, meta: meta::BufferMeta) -> Self {
        Self {
            id,
            position,
            buffer,
            meta,
        }
    }

    pub fn is_active(&self) -> bool {
        self.position() == 0
    }

    pub fn position(&self) -> usize {
        self.position
    }

    #[allow(unused)]
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn as_split(&self) -> (&buffer::Buffer, &meta::BufferMeta) {
        (&self.buffer, &self.meta)
    }

    pub fn as_split_mut<'a, 'b>(
        &'a mut self,
        ctrl: &'b mut Handle<State>,
    ) -> (emitter::buffer::Buffer<'a, 'b>, &'a mut meta::BufferMeta) {
        (
            emitter::buffer::Buffer::new(self.id, &mut self.buffer, ctrl),
            &mut self.meta,
        )
    }

    pub fn buffer(&self) -> &buffer::Buffer {
        &self.buffer
    }

    #[allow(unused)]
    pub fn buffer_mut<'a, 'b>(
        &'a mut self,
        ctrl: &'b mut Handle<State>,
    ) -> emitter::buffer::Buffer<'a, 'b> {
        emitter::buffer::Buffer::new(self.id, &mut self.buffer, ctrl)
    }

    pub fn meta(&self) -> &meta::BufferMeta {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut meta::BufferMeta {
        &mut self.meta
    }
}
