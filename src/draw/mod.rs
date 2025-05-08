use crate::{
    debug,
    rect::Rect,
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
            Color::None => Self::Reset,
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
    pub fn new(char: char, fg: Color, bg: Color) -> Self {
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
        Self::new(value.char, ANSIColor::from(value.fg))
    }
}

pub trait Surface {
    fn clear(&mut self);
    fn move_cursor(&mut self, point: Vec2<usize>);
    fn set(&mut self, position: Vec2<usize>, cell: Cell);
    fn dimensions(&self) -> Vec2<usize>;
}

pub trait WindowBind<'a> {
    fn bind(self, window: &'a mut window::Window) -> BoundedWindow<'a>;
}

impl<'a> WindowBind<'a> for Rect {
    fn bind(self, window: &'a mut window::Window) -> BoundedWindow<'a> {
        BoundedWindow {
            window,
            bound: self,
        }
    }
}

#[derive(Debug)]
pub struct BoundedWindow<'a> {
    window: &'a mut window::Window,
    bound: Rect,
}

impl Surface for BoundedWindow<'_> {
    fn set(&mut self, position: Vec2<usize>, cell: Cell) {
        let origin = self.bound.position();
        let new_pos = Vec2::new(position.x + origin.x, position.y + origin.y);
        window::Window::put_cell(self.window, new_pos, window::Cell::from(cell));
    }

    fn clear(&mut self) {
        let Vec2 {
            x: offs_x,
            y: offs_y,
        } = self.bound.position();
        let w = self.bound.width();
        let h = self.bound.height();
        for y in 0..h {
            for x in 0..w {
                let pos = Vec2::new(x + offs_x, y + offs_y);
                let _ = self.window.put_cell(pos, window::Cell::default());
            }
        }
    }

    fn move_cursor(&mut self, point: Vec2<usize>) {
        let origin = self.bound.position();
        let new_pos = Vec2::new(point.x + origin.x, point.y + origin.y);
        window::Window::set_cursor(self.window, new_pos);
    }

    fn dimensions(&self) -> Vec2<usize> {
        Vec2::new(self.bound.width(), self.bound.height())
    }
}
