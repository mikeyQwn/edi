//! Curcular buffer implementation

#[derive(Debug)]
pub struct CircularBuffer<const N: usize, T> {
    buffer: [T; N],
    write_head: usize,
    is_full: bool,
}

impl<const N: usize, T> Default for CircularBuffer<N, T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, T> CircularBuffer<N, T> {
    #[must_use]
    pub fn new() -> Self
    where
        T: Default + Copy,
    {
        Self {
            buffer: [T::default(); N],
            write_head: 0,
            is_full: false,
        }
    }

    pub fn new_from_fn(cb: impl FnMut(usize) -> T) -> Self {
        Self {
            buffer: core::array::from_fn(cb),
            write_head: 0,
            is_full: false,
        }
    }

    pub fn write(&mut self, value: T) {
        self.buffer[self.write_head] = value;
        self.advance();
    }

    pub fn write_indirect(&mut self, value: impl Into<T>) {
        self.write(value.into());
    }

    pub fn iter(&self) -> Iter<T> {
        if self.is_full {
            let (left, right) = self.buffer.split_at(self.write_head);
            Iter {
                left: right,
                right: left,
                index: 0,
                len: N,
            }
        } else {
            Iter {
                left: &self.buffer[..self.write_head],
                right: &[],
                index: 0,
                len: self.write_head,
            }
        }
    }

    fn advance(&mut self) {
        self.write_head += 1;
        if self.write_head >= N {
            self.write_head = 0;
            self.is_full = true;
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a CircularBuffer<N, T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Iter<'a, T> {
    left: &'a [T],
    right: &'a [T],
    index: usize,
    len: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let item = if self.index < self.left.len() {
            &self.left[self.index]
        } else {
            &self.right[self.index - self.left.len()]
        };

        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.len.saturating_sub(self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let buffer: CircularBuffer<5, i32> = CircularBuffer::new();
        assert_eq!(buffer.buffer, [0, 0, 0, 0, 0]);
        assert_eq!(buffer.write_head, 0);
        assert!(!buffer.is_full);
    }

    #[test]
    fn new_from_fn() {
        let buffer = CircularBuffer::<5, usize>::new_from_fn(|i| i * 2);
        assert_eq!(buffer.buffer, [0, 2, 4, 6, 8]);
    }

    #[test]
    fn iter() {
        let mut buffer = CircularBuffer::<3, i32>::new();
        buffer.write(1);
        buffer.write(2);
        let mut iter = buffer.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);

        buffer.write(3);
        buffer.write(4);
        buffer.write(5);
        let mut iter = buffer.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn write_and_iter_partially_filled() {
        let mut buffer = CircularBuffer::<5, i32>::new();
        buffer.write(1);
        buffer.write(2);
        buffer.write(3);

        let items: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(items, [1, 2, 3]);
    }

    #[test]
    fn write_and_iter_full_circle() {
        let mut buffer = CircularBuffer::<3, i32>::new();
        buffer.write(1);
        buffer.write(2);
        buffer.write(3);
        buffer.write(4); // overwrites 1

        let items: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(items, [2, 3, 4]);
    }

    #[test]
    fn multiple_wraps() {
        let mut buffer = CircularBuffer::<2, i32>::new();
        buffer.write(1);
        buffer.write(2);
        buffer.write(3);
        buffer.write(4);
        buffer.write(5);

        let items: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(items, [4, 5]);
    }

    #[test]
    fn exact_size_iterator() {
        let mut buffer = CircularBuffer::<4, i32>::new();
        buffer.write(1);
        buffer.write(2);

        let mut iter = buffer.iter();
        assert_eq!(iter.len(), 2);
        iter.next();
        assert_eq!(iter.len(), 1);
        iter.next();
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn empty_buffer_iter() {
        let buffer: CircularBuffer<3, i32> = CircularBuffer::new();
        let mut iter = buffer.iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn size_hint() {
        let mut buffer = CircularBuffer::<3, i32>::new();
        buffer.write(1);
        buffer.write(2);

        let iter = buffer.iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));
    }

    #[test]
    fn into_iter() {
        let mut buffer = CircularBuffer::<3, i32>::new();
        buffer.write(1);
        buffer.write(2);

        let items: Vec<_> = (&buffer).into_iter().copied().collect();
        assert_eq!(items, [1, 2]);
    }

    #[test]
    fn non_copy_type() {
        #[derive(Debug, PartialEq)]
        struct Data(String);

        let mut buffer = CircularBuffer::<2, Data>::new_from_fn(|_| Data(String::new()));
        buffer.write(Data("hello".into()));
        buffer.write(Data("world".into()));

        let items: Vec<_> = buffer.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, "hello");
        assert_eq!(items[1].0, "world");
    }

    #[test]
    fn edge_case_size_1() {
        let mut buffer = CircularBuffer::<1, i32>::new();
        buffer.write(10);
        assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), [10]);

        buffer.write(20);
        assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), [20]);
    }

    #[test]
    fn multiple_advances() {
        let mut buffer = CircularBuffer::<3, i32>::new();
        for i in 0..10 {
            buffer.write(i);
        }

        let items: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(items, [7, 8, 9]);
    }
}
