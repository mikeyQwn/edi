use std::collections::BTreeMap;

use edi::buffer::{self};
use edi_lib::brand::{Id, Tag};

use super::{buffer_bundle::BufferBundle, meta::BufferMeta};

#[derive(Debug)]
pub struct Buffers {
    brand: Tag,
    inner: BTreeMap<Id, BufferBundle>,
}

impl Buffers {
    #[must_use]
    pub fn new() -> Self {
        let brand = Tag::new();
        Self {
            brand,
            inner: BTreeMap::new(),
        }
    }

    pub fn first_mut(&mut self) -> Option<&mut BufferBundle> {
        self.inner.iter_mut().next().map(|(_, bundle)| bundle)
    }

    pub fn remove_first(&mut self) -> Option<BufferBundle> {
        let key = *self.inner.keys().next()?;
        self.inner.remove(&key)
    }

    pub fn attach(&mut self, buffer: buffer::Buffer, meta: BufferMeta) {
        let id = self.brand.child_id();
        self.inner.insert(id, BufferBundle::new(id, buffer, meta));
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BufferBundle> + DoubleEndedIterator {
        self.inner.values_mut()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
