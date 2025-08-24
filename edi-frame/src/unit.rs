use edi_term::coord::Dimensions;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Unit {
    Cells(usize),
    HeightRatio(f64),
    WidthRatio(f64),
    MinRatio(f64),
    MaxRatio(f64),
}

impl Unit {
    #[must_use]
    pub const fn zero() -> Self {
        Self::Cells(0)
    }

    #[must_use]
    pub const fn full_width() -> Self {
        Self::WidthRatio(1.0)
    }

    #[must_use]
    pub const fn full_height() -> Self {
        Self::HeightRatio(1.0)
    }

    #[must_use]
    pub const fn half_height() -> Self {
        Self::HeightRatio(0.5)
    }

    #[must_use]
    pub fn resolve(self, dimensions: Dimensions<usize>) -> usize {
        match self {
            Self::Cells(num) => num,
            Self::HeightRatio(factor) => Self::scale(dimensions.height, factor),
            Self::WidthRatio(factor) => Self::scale(dimensions.width, factor),
            Self::MinRatio(factor) => Self::scale(dimensions.width.min(dimensions.height), factor),
            Self::MaxRatio(factor) => Self::scale(dimensions.width.max(dimensions.height), factor),
        }
    }

    #[expect(clippy::cast_sign_loss, reason = "factor < 0 does not make sense")]
    #[expect(
        clippy::cast_precision_loss,
        reason = "nobody cares about the precision, we work with small values"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "terminal sizes are relatively small"
    )]
    const fn scale(value: usize, factor: f64) -> usize {
        (value as f64 * factor) as usize
    }
}
