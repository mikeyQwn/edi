use crate::vec2::Vec2;

// NOTE: make this generic if needed
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
}
