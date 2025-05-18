use crate::vec2::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    position: Vec2<usize>,
    width: usize,
    height: usize,
}

impl Rect {
    #[must_use]
    pub const fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            position: Vec2::new(x, y),
            width,
            height,
        }
    }

    #[must_use]
    pub const fn new_in_origin(width: usize, height: usize) -> Self {
        Self::new(0, 0, width, height)
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    #[must_use]
    pub const fn position(&self) -> Vec2<usize> {
        self.position
    }

    #[must_use]
    pub const fn contains_point(&self, point: Vec2<usize>) -> bool {
        if point.x < self.position.x || point.y < self.position.y {
            return false;
        }
        if point.x >= self.position.x + self.width || point.y >= self.position.y + self.height {
            return false;
        }
        true
    }

    #[must_use]
    pub const fn split_horizontal(&self, offset: usize) -> (Rect, Rect) {
        if offset > self.width {
            let zero_width = Rect::new(
                self.position.x + self.width,
                self.position.y,
                0,
                self.height,
            );
            return (*self, zero_width);
        }

        let left = Rect::new(self.position.x, self.position.y, offset, self.height);

        let right = Rect::new(
            self.position.x + offset,
            self.position.y,
            self.width.saturating_sub(offset),
            self.height,
        );

        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let rect = Rect::new(10, 20, 30, 40);
        assert_eq!(rect.position(), Vec2::new(10, 20));
        assert_eq!(rect.width(), 30);
        assert_eq!(rect.height(), 40);
    }

    #[test]
    fn new_in_origin() {
        let rect = Rect::new_in_origin(50, 60);
        assert_eq!(rect.position(), Vec2::new(0, 0));
        assert_eq!(rect.width(), 50);
        assert_eq!(rect.height(), 60);
    }

    #[test]
    fn contains_point() {
        let rect = Rect::new(10, 10, 20, 20);

        assert!(rect.contains_point(Vec2::new(10, 10)));
        assert!(rect.contains_point(Vec2::new(15, 15)));
        assert!(rect.contains_point(Vec2::new(29, 29)));

        assert!(!rect.contains_point(Vec2::new(9, 10)));
        assert!(!rect.contains_point(Vec2::new(10, 9)));
        assert!(!rect.contains_point(Vec2::new(30, 15)));
        assert!(!rect.contains_point(Vec2::new(15, 30)));
        assert!(!rect.contains_point(Vec2::new(30, 30)));
    }

    #[test]
    fn split_horizontal_normal() {
        let rect = Rect::new(5, 5, 10, 10);
        let (left, right) = rect.split_horizontal(4);

        assert_eq!(left.position(), Vec2::new(5, 5));
        assert_eq!(left.width(), 4);
        assert_eq!(left.height(), 10);

        assert_eq!(right.position(), Vec2::new(9, 5));
        assert_eq!(right.width(), 6); // 10 - 4
        assert_eq!(right.height(), 10);
    }

    #[test]
    fn split_horizontal_zero_offset() {
        let rect = Rect::new(5, 5, 10, 10);
        let (left, right) = rect.split_horizontal(0);

        assert_eq!(left.position(), Vec2::new(5, 5));
        assert_eq!(left.width(), 0);
        assert_eq!(left.height(), 10);

        assert_eq!(right.position(), Vec2::new(5, 5));
        assert_eq!(right.width(), 10);
        assert_eq!(right.height(), 10);
    }

    #[test]
    fn split_horizontal_full_width() {
        let rect = Rect::new(5, 5, 10, 10);
        let (left, right) = rect.split_horizontal(10);

        assert_eq!(left.position(), Vec2::new(5, 5));
        assert_eq!(left.width(), 10);
        assert_eq!(left.height(), 10);

        assert_eq!(right.position(), Vec2::new(15, 5));
        assert_eq!(right.width(), 0);
        assert_eq!(right.height(), 10);
    }

    #[test]
    fn split_horizontal_overflow() {
        let rect = Rect::new(5, 5, 10, 10);
        let (left, right) = rect.split_horizontal(15);

        assert_eq!(left.position(), rect.position());
        assert_eq!(left.width(), rect.width());
        assert_eq!(left.height(), rect.height());

        assert_eq!(right.position(), Vec2::new(15, 5));
        assert_eq!(right.width(), 0);
        assert_eq!(right.height(), 10);
    }

    #[test]
    fn split_horizontal_zero_size() {
        let rect = Rect::new(5, 5, 0, 0);
        let (left, right) = rect.split_horizontal(5);

        assert_eq!(left.position(), Vec2::new(5, 5));
        assert_eq!(left.width(), 0);
        assert_eq!(left.height(), 0);

        assert_eq!(right.position(), Vec2::new(5, 5));
        assert_eq!(right.width(), 0);
        assert_eq!(right.height(), 0);
    }
}
