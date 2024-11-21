use std::io::{stdout, Write};

use crate::{
    escaping::{ANSIColor, EscapeBuilder},
    vec2::Vec2,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    character: char,
    color: ANSIColor,
}

impl Cell {
    #[must_use]
    pub const fn new(character: char, color: ANSIColor) -> Self {
        Self { character, color }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(' ', ANSIColor::Red)
    }
}

pub struct Window {
    width: usize,
    height: usize,

    cursor_pos: Vec2<usize>,

    buffer: Vec<Cell>,
    back_buffer: Vec<Cell>,
}

impl Window {
    pub const fn new() -> Self {
        Self {
            width: 0,
            height: 0,

            cursor_pos: Vec2::new(0, 0),

            buffer: Vec::new(),
            back_buffer: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        // TODO: Keep the window state maybe
        self.width = width;
        self.height = height;
        self.buffer = vec![Cell::default(); width * height];
        self.back_buffer = vec![Cell::default(); width * height];
    }

    pub fn render(&mut self) -> Result<(), std::io::Error> {
        let diffs = self.produce_diffs();
        self.buffer.copy_from_slice(&self.back_buffer);

        stdout().write_all(diffs.build().as_bytes())?;
        stdout().flush()
    }

    pub fn render_cursor(&mut self) -> Result<(), std::io::Error> {
        stdout().write_all(
            EscapeBuilder::new()
                .move_to(self.cursor_pos)
                .build()
                .as_bytes(),
        )?;
        stdout().flush()
    }

    pub fn set_cursor(&mut self, new_pos: Vec2<usize>) {
        self.cursor_pos = new_pos;
    }

    pub fn rerender(&mut self) -> Result<(), std::io::Error> {
        self.buffer.copy_from_slice(&self.back_buffer);
        let string_diffs = self.as_string();
        let changes = EscapeBuilder::new()
            .clear_screen()
            .write(string_diffs.into())
            .move_to(Vec2::new(0, 0))
            .build();
        stdout().write_all(changes.as_bytes())?;
        stdout().flush()
    }

    pub fn produce_diffs<'a>(&self) -> EscapeBuilder<'a> {
        let mut escape = EscapeBuilder::new();

        let mut prev_pos = None;

        let mut prev_color = None;
        for y in 0..self.height {
            let row_offs = y * self.width;
            for x in 0..self.width {
                let index = row_offs + x;
                let cell = self.back_buffer[index];
                if cell != self.buffer[index] {
                    if prev_pos != Some((x.saturating_sub(1), y)) {
                        escape = escape.move_to(Vec2::new(x, y));
                    }

                    if prev_color != Some(cell.color) {
                        prev_color = Some(cell.color);
                        escape = escape.set_color(cell.color);
                    }

                    prev_pos = Some((x, y));
                    escape = escape.write(cell.character.to_string().into());
                }
            }
        }

        escape = escape.move_to(self.cursor_pos);

        escape
    }

    pub fn put_cell(&mut self, pos: Vec2<usize>, cell: Cell) -> bool {
        if pos.x >= self.width || pos.y >= self.height {
            return false;
        }

        if cell.character.is_control() {
            return false;
        }

        let index = pos.y * self.width + pos.x;
        self.back_buffer[index] = cell;

        true
    }

    fn as_string(&self) -> String {
        let mut result = EscapeBuilder::new();

        for i in 0..self.height {
            for j in 0..self.width {
                let index = i * self.width + j;
                let mut prev_cell = None;
                let cell = self.buffer[index];
                if index != 0 {
                    prev_cell = self.buffer.get(index - 1);
                }
                if prev_cell.map(|c| c.color) != Some(cell.color) {
                    result = result.set_color(cell.color);
                }
                result = result.write(cell.character.to_string().into());
            }
        }

        result.build()
    }
}

#[cfg(test)]
mod tests {}
