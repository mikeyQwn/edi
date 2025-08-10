use std::sync::atomic::{AtomicUsize, Ordering};

static BRAND: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Tag(usize, usize);

impl Tag {
    pub fn new() -> Self {
        Self(BRAND.fetch_add(1, Ordering::Relaxed), 0)
    }

    pub fn child_id(&mut self) -> Id {
        let id = self.1;
        self.1 += 1;
        Id(self.0, id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(usize, usize);
