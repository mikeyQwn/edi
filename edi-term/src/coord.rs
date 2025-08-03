//! Coordinate/dimensions utilities

/// Screenspace coordinates
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord {
    /// The `x` coordinate
    pub x: usize,
    /// The `y` coordinate
    pub y: usize,
}

impl Coord {
    /// Initializes `Coord` with given `x` and `y`
    #[must_use]
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// Screenspace dimensions
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dimensions<T> {
    /// Width of the window/screen/terminal
    pub width: T,
    /// Height of the window/screen/terminal
    pub height: T,
}

impl<T> Dimensions<T> {
    /// Initializes `Dimensions` with given `width` and `height`
    #[must_use]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    /// Maps `x` and `y` to `U` using given function `f`
    #[must_use]
    pub fn map<U, F>(self, mut f: F) -> Dimensions<U>
    where
        F: FnMut(T) -> U,
    {
        Dimensions::new(f(self.width), f(self.height))
    }
}
