macro_rules! impl_vec2_ops {
    ($trait:ident, $method:ident, $($bound:tt)+) => {
        impl<T> std::ops::$trait for Vec2<T>
        where
            T: $($bound)+,
        {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                Self::new(self.x.$method(rhs.x), self.y.$method(rhs.y))
            }
        }
    };
}

macro_rules! impl_vec2_ops_assign {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    #[must_use]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl_vec2_ops!(Add, add, std::ops::Add<Output = T>);
impl_vec2_ops!(Sub, sub, std::ops::Sub<Output = T>);
impl_vec2_ops!(Mul, mul, std::ops::Mul<Output = T> + Copy);
impl_vec2_ops!(Div, div, std::ops::Div<Output = T> + Copy);
impl_vec2_ops!(Rem, rem, std::ops::Rem<Output = T> + Copy);
impl_vec2_ops!(BitAnd, bitand, std::ops::BitAnd<Output = T> + Copy);
impl_vec2_ops!(BitOr, bitor, std::ops::BitOr<Output = T> + Copy);
impl_vec2_ops!(BitXor, bitxor, std::ops::BitXor<Output = T> + Copy);
impl_vec2_ops!(Shl, shl, std::ops::Shl<Output = T> + Copy);
impl_vec2_ops!(Shr, shr, std::ops::Shr<Output = T> + Copy);

impl_vec2_ops_assign!(AddAssign, add_assign, std::ops::AddAssign);
impl_vec2_ops_assign!(SubAssign, sub_assign, std::ops::SubAssign);
impl_vec2_ops_assign!(MulAssign, mul_assign, std::ops::MulAssign + Copy);
impl_vec2_ops_assign!(DivAssign, div_assign, std::ops::DivAssign + Copy);
impl_vec2_ops_assign!(RemAssign, rem_assign, std::ops::RemAssign + Copy);
impl_vec2_ops_assign!(BitAndAssign, bitand_assign, std::ops::BitAndAssign + Copy);
impl_vec2_ops_assign!(BitOrAssign, bitor_assign, std::ops::BitOrAssign + Copy);
impl_vec2_ops_assign!(BitXorAssign, bitxor_assign, std::ops::BitXorAssign + Copy);
impl_vec2_ops_assign!(ShlAssign, shl_assign, std::ops::ShlAssign + Copy);
impl_vec2_ops_assign!(ShrAssign, shr_assign, std::ops::ShrAssign + Copy);
