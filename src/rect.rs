use crate::vec2::Vec2;

// NOTE: make this generic if needed
pub struct Rect {
    origin: Vec2<usize>,
    width: usize,
    height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            origin: Vec2::new(x, y),
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn origin(&self) -> Vec2<usize> {
        self.origin
    }
}
