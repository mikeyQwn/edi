use edi_term::{
    coord::{Coord, Dimensions, UDims},
    escaping::ANSIColor,
    window,
};

use crate::{
    cell::{Cell, Color},
    rect::Rect,
};

/// A generic terminal-like surface that can be drawn to
pub trait Surface {
    fn clear(&mut self, color: Color);

    fn move_cursor(&mut self, point: Coord);
    fn set(&mut self, position: Coord, cell: Cell);

    fn dimensions(&self) -> UDims;
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
    fn clear(&self, surface: &mut S, color: Color);
    fn move_cursor(&self, point: Coord, surface: &mut S);
    fn set(&self, position: Coord, cell: Cell, surface: &mut S);
    fn dimensions(&self, surface: &S) -> Dimensions<usize>;
}

fn get_bounded_position(position: Coord, bound: &Rect) -> Option<Coord> {
    let bound_position = bound.position();
    let position = Coord::new(position.x + bound_position.x, position.y + bound_position.y);
    bound.contains_point(position).then_some(position)
}

impl<S> BoundExt<S> for Rect
where
    S: Surface,
{
    fn set(&self, position: Coord, cell: Cell, surface: &mut S) {
        let Some(position) = get_bounded_position(position, self) else {
            return;
        };

        surface.set(position, cell);
    }

    fn clear(&self, surface: &mut S, color: Color) {
        let w = self.width();
        let h = self.height();
        for y in 0..h {
            for x in 0..w {
                let Some(position) = get_bounded_position(Coord::new(x, y), self) else {
                    continue;
                };

                let cell = Cell::new(' ', Color::None, color);
                surface.set(position, cell);
            }
        }
    }

    fn dimensions(&self, surface: &S) -> Dimensions<usize> {
        let surface_dimensions = surface.dimensions();
        let position = self.position();

        Dimensions::new(
            self.width()
                .min(surface_dimensions.width.saturating_sub(position.x)),
            self.height()
                .midpoint(surface_dimensions.height.saturating_sub(position.y)),
        )
    }

    fn move_cursor(&self, point: Coord, surface: &mut S) {
        let Some(position) = get_bounded_position(point, self) else {
            return;
        };

        surface.move_cursor(position);
    }
}

impl Surface for window::Window {
    fn set(&mut self, position: Coord, cell: Cell) {
        window::Window::put_cell(self, position, window::Cell::from(cell));
    }

    fn clear(&mut self, color: Color) {
        window::Window::clear(self, ANSIColor::from(color));
    }

    fn dimensions(&self) -> Dimensions<usize> {
        window::Window::size(self)
    }

    fn move_cursor(&mut self, point: Coord) {
        window::Window::set_cursor(self, point);
    }
}

#[derive(Debug)]
pub struct BoundedWindow<'a> {
    window: &'a mut window::Window,
    bound: Rect,
}

impl Surface for BoundedWindow<'_> {
    fn set(&mut self, position: Coord, cell: Cell) {
        self.bound.set(position, cell, self.window);
    }

    fn clear(&mut self, color: Color) {
        self.bound.clear(self.window, color);
    }

    fn move_cursor(&mut self, point: Coord) {
        self.bound.move_cursor(point, self.window);
    }

    fn dimensions(&self) -> Dimensions<usize> {
        self.bound.dimensions(self.window)
    }
}
