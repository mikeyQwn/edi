//! All methods that mutate buffer's inner string

use std::collections::VecDeque;

use super::Buffer;

#[derive(Debug, PartialEq, Eq)]
enum Change {
    Write { at: usize },
    Delete { range: std::ops::Range<usize> },
}

pub(crate) struct ChangeHistory {
    inner: VecDeque<Change>,
    capacity: usize,
}

impl ChangeHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn write(&mut self, change: Change) {
        if self.inner.len() == self.capacity {
            self.inner.pop_front();
        }
        self.inner.push_back(change);
    }

    pub fn pop_change(&mut self) -> Option<Change> {
        self.inner.pop_back()
    }
}

impl Buffer {
    pub fn write(&mut self, c: char) {
        let is_empty = self.inner.is_empty();
        self.inner
            .insert(self.cursor_offset, c.to_string().as_ref());

        if c == '\n' {
            self.cursor_offset += 1;
            return;
        }

        if is_empty {
            self.inner.insert(self.cursor_offset, "\n");
        }

        self.cursor_offset += 1;
    }

    pub fn delete(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            self.inner.delete(self.cursor_offset..=self.cursor_offset);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn change_history_write() {
        let mut ch = ChangeHistory::new(4);
        ch.write(Change::Write { at: 0 });
        ch.write(Change::Write { at: 1 });
        ch.write(Change::Write { at: 2 });
        ch.write(Change::Write { at: 3 });
        ch.write(Change::Write { at: 4 });

        assert_eq!(ch.pop_change().unwrap(), Change::Write { at: 4 });
        assert_eq!(ch.pop_change().unwrap(), Change::Write { at: 3 });
        assert_eq!(ch.pop_change().unwrap(), Change::Write { at: 2 });
        assert_eq!(ch.pop_change().unwrap(), Change::Write { at: 1 });
        assert!(ch.pop_change().is_none());
    }
}
