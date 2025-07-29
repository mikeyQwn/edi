use edi_lib::vec2::Vec2;
use edi_term::{escaping::ANSIColor, window};

use crate::rect::Rect;

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

/// A generic terminal-like surface that can be drawn to
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

/// An extension trait that adds bounding capabilities to `Rect`
pub trait BoundExt<S>
where
    S: Surface,
{
    fn clear(&self, surface: &mut S);
    fn move_cursor(&self, point: Vec2<usize>, surface: &mut S);
    fn set(&self, position: Vec2<usize>, cell: Cell, surface: &mut S);
    fn dimensions(&self, surface: &S) -> Vec2<usize>;
}

fn get_bounded_position(position: Vec2<usize>, bound: &Rect) -> Option<Vec2<usize>> {
    let bound_position = bound.position();
    let position = Vec2::new(position.x + bound_position.x, position.y + bound_position.y);
    bound.contains_point(position).then_some(position)
}

impl<S> BoundExt<S> for Rect
where
    S: Surface,
{
    fn set(&self, position: Vec2<usize>, cell: Cell, surface: &mut S) {
        let Some(position) = get_bounded_position(position, self) else {
            return;
        };

        surface.set(position, cell);
    }

    fn clear(&self, surface: &mut S) {
        let w = self.width();
        let h = self.height();
        for y in 0..h {
            for x in 0..w {
                let Some(position) = get_bounded_position(Vec2::new(x, y), self) else {
                    continue;
                };

                surface.set(position, Cell::default());
            }
        }
    }

    fn dimensions(&self, surface: &S) -> Vec2<usize> {
        let surface_dimensions = surface.dimensions();
        let position = self.position();

        Vec2::new(
            self.width()
                .min(surface_dimensions.x.saturating_sub(position.x)),
            self.height()
                .midpoint(surface_dimensions.y.saturating_sub(position.y)),
        )
    }

    fn move_cursor(&self, point: Vec2<usize>, surface: &mut S) {
        let Some(position) = get_bounded_position(point, self) else {
            return;
        };

        surface.move_cursor(position);
    }
}

impl Surface for window::Window {
    fn set(&mut self, position: Vec2<usize>, cell: Cell) {
        window::Window::put_cell(self, position.as_coords(), window::Cell::from(cell));
    }

    fn clear(&mut self) {
        window::Window::clear(self);
    }

    fn dimensions(&self) -> Vec2<usize> {
        Vec2::from_dims(window::Window::size(self))
    }

    fn move_cursor(&mut self, point: Vec2<usize>) {
        window::Window::set_cursor(self, point.as_coords());
    }
}

#[derive(Debug)]
pub struct BoundedWindow<'a> {
    window: &'a mut window::Window,
    bound: Rect,
}

impl Surface for BoundedWindow<'_> {
    fn set(&mut self, position: Vec2<usize>, cell: Cell) {
        self.bound.set(position, cell, self.window);
    }

    fn clear(&mut self) {
        self.bound.clear(self.window);
    }

    fn move_cursor(&mut self, point: Vec2<usize>) {
        self.bound.move_cursor(point, self.window);
    }

    fn dimensions(&self) -> Vec2<usize> {
        self.bound.dimensions(self.window)
    }
}
