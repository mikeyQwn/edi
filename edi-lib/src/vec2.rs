//! A two-element vector

use edi_term::coord::{Coord, Dimensions};

/// A two-element vector, usually is used to represent 2d coordinates
///
/// Operations on the `Vec2` are performed for both elements respectively
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Vec2<T> {
    /// First element
    pub x: T,
    /// Second element
    pub y: T,
}

impl<T> Vec2<T> {
    /// Instantiates a vector of two elements
    #[must_use]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn map<U, F>(self, mut f: F) -> Vec2<U>
    where
        F: FnMut(T) -> U,
    {
        Vec2::new(f(self.x), f(self.y))
    }

    #[must_use]
    pub fn from_dims(Dimensions { width, height }: Dimensions<T>) -> Self {
        Self::new(width, height)
    }

    #[must_use]
    pub fn into_dims(self) -> Dimensions<T> {
        Dimensions::new(self.x, self.y)
    }
}

impl Vec2<usize> {
    #[must_use]
    pub const fn as_coords(&self) -> Coord {
        Coord::new(self.x, self.y)
    }
}

macro_rules! impl_ops {
    ($trait:ident, $method:ident, $($bound:tt)+) => {
        impl<T, U> std::ops::$trait for Vec2<T>
        where
            T: $($bound)+,
        {
            type Output = Vec2<U>;

            fn $method(self, rhs: Self) -> Self::Output {
                Vec2::new(self.x.$method(rhs.x), self.y.$method(rhs.y))
            }
        }
    };
}

macro_rules! impl_ops_assign {
    ($trait:ident, $method:ident, $($bound:tt)+) => {
        impl<T> std::ops::$trait for Vec2<T>
        where
            T: $($bound)+,
        {
            fn $method(&mut self, rhs: Self) {
                self.x.$method(rhs.x);
                self.y.$method(rhs.y);
            }
        }
    };
}

impl_ops!(Add, add, std::ops::Add<Output = U>);
impl_ops!(Sub, sub, std::ops::Sub<Output = U>);
impl_ops!(Mul, mul, std::ops::Mul<Output = U>);
impl_ops!(Div, div, std::ops::Div<Output = U>);
impl_ops!(Rem, rem, std::ops::Rem<Output = U>);
impl_ops!(BitAnd, bitand, std::ops::BitAnd<Output = U>);
impl_ops!(BitOr, bitor, std::ops::BitOr<Output = U>);
impl_ops!(BitXor, bitxor, std::ops::BitXor<Output = U>);
impl_ops!(Shl, shl, std::ops::Shl<Output = U>);
impl_ops!(Shr, shr, std::ops::Shr<Output = U>);

impl_ops_assign!(AddAssign, add_assign, std::ops::AddAssign);
impl_ops_assign!(SubAssign, sub_assign, std::ops::SubAssign);
impl_ops_assign!(MulAssign, mul_assign, std::ops::MulAssign);
impl_ops_assign!(DivAssign, div_assign, std::ops::DivAssign);
impl_ops_assign!(RemAssign, rem_assign, std::ops::RemAssign);
impl_ops_assign!(BitAndAssign, bitand_assign, std::ops::BitAndAssign);
impl_ops_assign!(BitOrAssign, bitor_assign, std::ops::BitOrAssign);
impl_ops_assign!(BitXorAssign, bitxor_assign, std::ops::BitXorAssign);
impl_ops_assign!(ShlAssign, shl_assign, std::ops::ShlAssign);
impl_ops_assign!(ShrAssign, shr_assign, std::ops::ShrAssign);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations() {
        let mut a = Vec2::new(10_isize, 5_isize);
        let b = Vec2::new(5_isize, 6_isize);

        assert_eq!(a - b, Vec2::new(5_isize, -1_isize));
        a -= b;

        assert_eq!(a, Vec2::new(5_isize, -1_isize));
    }
}
