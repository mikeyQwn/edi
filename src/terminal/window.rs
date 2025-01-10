//! An api for handling the raw mode terminal

use std::io::{stdout, Result, Stdout, Write};

use crate::{
    escaping::{ANSIColor, EscapeBuilder},
    vec2::Vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    character: char,
    fg_color: ANSIColor,
}

impl Cell {
    #[must_use]
    pub const fn new(character: char, fg_color: ANSIColor) -> Self {
        Self {
            character,
            fg_color,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(' ', ANSIColor::Red)
    }
}

/// A TUI "Window"
///
/// It is used for drawing in the terminal that is exactly the size of the window
/// The user is responsible for resizing the `Window` when necessary with the `set_size` method
#[derive(Debug)]
pub struct Window<W = Stdout>
where
    W: Write,
{
    width: usize,
    height: usize,

    cursor_pos: Vec2<usize>,

    buffer: Vec<Cell>,
    back_buffer: Vec<Cell>,

    writer: W,
}

impl<W> Window<W>
where
    W: Write,
{
    pub fn from_writer(writer: W) -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),

            cursor_pos: Vec2::default(),

            buffer: Vec::default(),
            back_buffer: Vec::default(),

            writer,
        }
    }
}

impl Window {
    pub fn new() -> Self {
        Self::from_writer(stdout())
    }
}

impl<W> Window<W>
where
    W: Write,
{
    pub fn set_size(&mut self, Vec2 { x, y }: Vec2<usize>) {
        self.width = x;
        self.height = y;

        self.buffer = vec![Cell::default(); x * y];
        self.back_buffer = self.buffer.clone();
    }

    pub fn render(&mut self) -> Result<()> {
        let diffs = self.produce_diffs();
        self.buffer.copy_from_slice(&self.back_buffer);
        self.write_flush(diffs.build().as_bytes())
    }

    pub fn clear(&mut self) {
        self.back_buffer = vec![Cell::default(); self.width * self.height];
    }

    pub fn set_cursor(&mut self, new_pos: Vec2<usize>) {
        self.cursor_pos = new_pos;
    }

    pub fn rerender(&mut self) -> Result<()> {
        self.buffer.copy_from_slice(&self.back_buffer);
        let string_diffs = self.as_string();
        let changes = EscapeBuilder::new()
            .clear_screen()
            .write(string_diffs.into())
            .move_to(Vec2::default())
            .build();

        self.write_flush(changes.as_bytes())
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
                if cell == self.buffer[index] {
                    continue;
                }

                if prev_pos != Some((x.saturating_sub(1), y)) {
                    escape = escape.move_to(Vec2::new(x, y));
                }

                if prev_color != Some(cell.fg_color) {
                    prev_color = Some(cell.fg_color);
                    escape = escape.set_color(cell.fg_color);
                }

                prev_pos = Some((x, y));
                escape = escape.write(cell.character.to_string().into());
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
                if prev_cell.map(|c| c.fg_color) != Some(cell.fg_color) {
                    result = result.set_color(cell.fg_color);
                }
                result = result.write(cell.character.to_string().into());
            }
        }

        result.build()
    }

    fn write_flush(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf)?;
        self.writer.flush()
    }
}
