use std::collections::BTreeMap;

use edi_lib::brand::{Id, Tag};
use edi_lib::buffer::{self};

use super::{buffer_bundle::BufferBundle, meta::BufferMeta, Mode};

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Selector {
    First,
    Active,
    Nth(usize),
    WithId(Id),
}

#[derive(Debug)]
pub struct Buffers {
    brand: Tag,
    inner: BTreeMap<Id, BufferBundle>,
    buffer_order: Vec<Id>, // INVARIANT: length is always equal to the length of
                           // inner AND all ids are unique
}

impl Buffers {
    #[must_use]
    pub fn new() -> Self {
        let brand = Tag::new();
        Self {
            brand,
            inner: BTreeMap::new(),
            buffer_order: Vec::new(),
        }
    }

    #[allow(unused)]
    pub fn active_buffer_mode(&self) -> Option<Mode> {
        self.active().map(BufferBundle::meta).map(BufferMeta::mode)
    }

    #[allow(unused)]
    pub fn get(&self, selector: &Selector) -> Option<&BufferBundle> {
        match selector {
            Selector::First | Selector::Nth(0) => self.first(),
            Selector::Active => self.active(),
            Selector::WithId(id) => self.inner.get(id),
            &Selector::Nth(n) => self.nth(n),
        }
    }

    pub fn get_mut(&mut self, selector: &Selector) -> Option<&mut BufferBundle> {
        match selector {
            Selector::First | Selector::Nth(0) => self.first_mut(),
            Selector::Active => self.active_mut(),
            Selector::WithId(id) => self.inner.get_mut(id),
            &Selector::Nth(n) => self.nth_mut(n),
        }
    }

    pub fn first(&self) -> Option<&BufferBundle> {
        let first_id = self.buffer_order.first()?;
        self.inner.get(first_id)
    }

    pub fn second(&self) -> Option<&BufferBundle> {
        let first_id = self.buffer_order.get(1)?;
        self.inner.get(first_id)
    }

    #[allow(unused)]
    pub fn nth(&self, n: usize) -> Option<&BufferBundle> {
        let id = self.buffer_order.get(n)?;
        self.inner.get(id)
    }

    pub fn active(&self) -> Option<&BufferBundle> {
        self.first()
    }

    pub fn first_mut(&mut self) -> Option<&mut BufferBundle> {
        let first_id = self.buffer_order.first()?;
        self.inner.get_mut(first_id)
    }

    pub fn active_mut(&mut self) -> Option<&mut BufferBundle> {
        self.first_mut()
    }

    pub fn nth_mut(&mut self, n: usize) -> Option<&mut BufferBundle> {
        let id = self.buffer_order.get(n)?;
        self.inner.get_mut(id)
    }

    pub fn remove(&mut self, id: Id) -> Option<BufferBundle> {
        let bundle = self.inner.remove(&id)?;
        let pos = bundle.position();

        self.buffer_order.remove(pos);
        for i in pos..self.buffer_order.len() {
            self.set_buffer_order(i);
        }

        Some(bundle)
    }

    #[allow(unused)]
    pub fn remove_first(&mut self) -> Option<BufferBundle> {
        (!self.buffer_order.is_empty()).then_some(())?;
        let first_id = self.buffer_order.remove(0);
        self.inner.remove(&first_id)
    }

    pub fn attach(&mut self, buffer: buffer::Buffer, meta: BufferMeta) {
        let id = self.brand.child_id();
        self.inner.insert(
            id,
            BufferBundle::new(id, self.buffer_order.len(), buffer, meta),
        );
        self.buffer_order.push(id);
    }

    pub fn attach_first(&mut self, buffer: buffer::Buffer, meta: BufferMeta) {
        self.attach(buffer, meta);
        self.swap(0, self.inner.len() - 1);
    }

    fn swap(&mut self, a_ord: usize, b_ord: usize) {
        self.buffer_order.swap(a_ord, b_ord);
        self.set_buffer_order(a_ord);
        self.set_buffer_order(b_ord);
    }

    fn set_buffer_order(&mut self, order: usize) {
        if let Some(entry) = self.inner.get_mut(&self.buffer_order[order]) {
            entry.position = order;
        }
    }

    #[allow(unused)]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &BufferBundle> {
        BuffersIter::new(&self.inner, &self.buffer_order)
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut BufferBundle> {
        // SAFETY: this struct upholds the invariant that all ids in buffer_order are unique
        //
        // TODO: try to use some crate for this, get rid of unsafe
        unsafe { BuffersIterMut::new(&mut self.inner, &self.buffer_order) }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

struct BuffersIter<'a> {
    inner: &'a BTreeMap<Id, BufferBundle>,
    order: std::slice::Iter<'a, Id>,
}

impl<'a> Iterator for BuffersIter<'a> {
    type Item = &'a BufferBundle;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.order.next()?;
        self.inner.get(id)
    }
}

impl DoubleEndedIterator for BuffersIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let id = self.order.next_back()?;
        self.inner.get(id)
    }
}

#[allow(unused)]
impl<'a> BuffersIter<'a> {
    #[must_use]
    pub fn new(inner: &'a BTreeMap<Id, BufferBundle>, buffer_order: &'a [Id]) -> Self {
        Self {
            inner,
            order: buffer_order.iter(),
        }
    }
}

struct BuffersIterMut<'a> {
    inner: *mut BTreeMap<Id, BufferBundle>,
    order: std::slice::Iter<'a, Id>,
}

impl<'a> BuffersIterMut<'a> {
    /// SAFETY: for this iterator to be safe, the caller must guarantee that
    /// buffer order has no repeating Ids
    unsafe fn new(inner: &'a mut BTreeMap<Id, BufferBundle>, buffer_order: &'a [Id]) -> Self {
        Self {
            inner,
            order: buffer_order.iter(),
        }
    }
}

impl<'a> Iterator for BuffersIterMut<'a> {
    type Item = &'a mut BufferBundle;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.order.next()?;
        // SAFETY: we guarantee that the order does not repeat the
        // ids, so there is no way for self.inner.get_mut to alias the same data
        unsafe { &mut *self.inner }.get_mut(id)
    }
}

impl DoubleEndedIterator for BuffersIterMut<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let id = self.order.next_back()?;
        // SAFETY: we guarantee that the order does not repeat the
        // ids, so there is no way for self.inner.get_mut to alias the same data
        unsafe { &mut *self.inner }.get_mut(id)
    }
}

#[cfg(test)]
mod tests {
    use edi_frame::unit::Unit;
    use edi_lib::vec2::Vec2;

    use crate::app::{meta, Mode};

    use super::*;

    fn make_buffers(n: usize) -> Buffers {
        let mut bufs = Buffers::new();
        for _ in 0..n {
            bufs.attach(buffer::Buffer::new(""), meta::BufferMeta::new(Mode::Normal));
        }
        bufs
    }

    #[test]
    fn new_is_empty() {
        let mut b = Buffers::new();
        assert_eq!(b.len(), 0);
        assert!(b.active_mut().is_none());
        assert!(b.buffer_order.is_empty());
    }

    #[test]
    fn attach_increases_len_and_order() {
        let b = make_buffers(3);
        assert_eq!(b.len(), 3);
        assert_eq!(b.buffer_order.len(), 3);
        assert_eq!(b.inner.len(), 3);
    }

    #[test]
    fn first_mut_returns_first_element() {
        let mut b = make_buffers(2);
        let first_id = b.buffer_order[0];
        let first = b.active_mut().unwrap();
        assert_eq!(first.id(), first_id);
    }

    #[test]
    fn remove_first_removes_in_order() {
        let mut b = make_buffers(3);
        let first_id = b.buffer_order[0];
        let removed = b.remove_first().unwrap();
        assert_eq!(removed.id(), first_id);
        assert_eq!(b.len(), 2);
        assert_eq!(b.buffer_order.len(), 2);
        assert_eq!(b.inner.len(), 2);
    }

    #[test]
    fn remove_first_on_empty_returns_none() {
        let mut b = Buffers::new();
        assert!(b.remove_first().is_none());
    }

    #[test]
    fn attach_first_places_element_at_front() {
        let mut b = make_buffers(2);
        b.attach_first(buffer::Buffer::new(""), BufferMeta::new(Mode::Normal));

        let first = b.active_mut().unwrap();
        assert_eq!(first.id(), b.buffer_order[0]);
    }

    #[test]
    fn iter_mut_yields_all_in_order() {
        let mut b = make_buffers(3);
        let ids = b.buffer_order.clone();
        let iter_ids: Vec<Id> = b.iter_mut().map(|bundle| bundle.id()).collect();
        assert_eq!(ids, iter_ids);
    }

    #[test]
    fn iter_mut_double_ended() {
        let mut b = make_buffers(3);
        let mut it = b.iter_mut();
        let front = it.next().unwrap().id();
        let back = it.next_back().unwrap().id();
        drop(it);
        assert_eq!(front, b.buffer_order[0]);
        assert_eq!(back, b.buffer_order[2]);
    }

    #[test]
    fn iter_mut_allows_mutation() {
        let mut b = make_buffers(2);
        for bundle in b.iter_mut() {
            let meta = bundle.meta_mut();
            meta.size = Vec2::new(Unit::Cells(1), Unit::Cells(1));
        }
        for bundle in b.iter_mut() {
            assert_eq!(
                bundle.meta_mut().size,
                Vec2::new(Unit::Cells(1), Unit::Cells(1))
            );
        }
    }

    #[test]
    fn order_matches_inner_len() {
        let mut b = make_buffers(5);
        assert_eq!(b.inner.len(), b.buffer_order.len());
        b.remove_first();
        assert_eq!(b.inner.len(), b.buffer_order.len());
    }
}
