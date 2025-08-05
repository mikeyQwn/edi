use edi_term::{escaping::ANSIColor, window};

#[allow(unused)]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    #[default]
    White,

    None,
}

impl From<ANSIColor> for Color {
    fn from(value: ANSIColor) -> Self {
        match value {
            ANSIColor::Black => Color::Black,
            ANSIColor::Red => Color::Red,
            ANSIColor::Green => Color::Green,
            ANSIColor::Yellow => Color::Yellow,
            ANSIColor::Blue => Color::Blue,
            ANSIColor::Magenta => Color::Magenta,
            ANSIColor::Cyan => Color::Cyan,
            ANSIColor::White => Color::White,
            _ => Color::default(),
        }
    }
}

impl From<Color> for ANSIColor {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => Self::Black,
            Color::Red => Self::Red,
            Color::Green => Self::Green,
            Color::Yellow => Self::Yellow,
            Color::Blue => Self::Blue,
            Color::Magenta => Self::Magenta,
            Color::Cyan => Self::Cyan,
            Color::White => Self::White,
            Color::None => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub char: char,
    pub fg: Color,
    pub bg: Color,
}

impl Cell {
    #[must_use]
    pub const fn new(char: char, fg: Color, bg: Color) -> Self {
        Self { char, fg, bg }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(' ', Color::White, Color::None)
    }
}

impl From<window::Cell> for Cell {
    fn from(value: window::Cell) -> Self {
        Self {
            char: value.character,
            fg: Color::from(value.fg_color),
            ..Default::default()
        }
    }
}

impl From<Cell> for window::Cell {
    fn from(value: Cell) -> Self {
        Self::new(
            value.char,
            ANSIColor::from(value.fg),
            ANSIColor::from(value.bg),
        )
    }
}
