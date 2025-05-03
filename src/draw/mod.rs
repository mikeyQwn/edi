use crate::{
    terminal::{escaping::ANSIColor, window},
    vec2::Vec2,
};

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
        }
    }
}

#[derive(Debug, Default)]
pub struct Cell {
    char: char,
    fg: Color,
    bg: Color,
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
        Self::new(value.char, ANSIColor::from(value.fg))
    }
}

pub trait Surface {
    fn clear(&mut self);
    fn move_cursor(&mut self, point: Vec2<usize>);
    fn set(&mut self, position: Vec2<usize>, cell: Cell);
}

impl Surface for window::Window {
    fn set(&mut self, position: Vec2<usize>, cell: Cell) {
        window::Window::put_cell(self, position, window::Cell::from(cell));
    }

    fn clear(&mut self) {
        window::Window::clear(self);
    }

    fn move_cursor(&mut self, point: Vec2<usize>) {
        window::Window::set_cursor(self, point);
    }
}
