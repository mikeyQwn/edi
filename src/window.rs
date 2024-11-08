use std::io::{stdout, Write};

use crate::{escaping::EscapeBuilder, terminal::Terminal};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    character: char,
}

impl Default for Cell {
    fn default() -> Self {
        Self { character: ' ' }
    }
}

pub struct Window {
    width: usize,
    height: usize,

    cursor_pos: (usize, usize),
    prev_cursor_pos: (usize, usize),

    buffer: Vec<Cell>,
    back_buffer: Vec<Cell>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,

            cursor_pos: (0, 0),
            prev_cursor_pos: (0, 0),

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
        Terminal::set_position(0, 0)?;
        let diffs = self.produce_diffs();
        self.buffer.copy_from_slice(&self.back_buffer);

        stdout().write_all(diffs.as_bytes())?;
        if self.cursor_pos != self.prev_cursor_pos {
            Terminal::set_position(self.cursor_pos.0, self.cursor_pos.1)?;
        }
        stdout().flush()
    }

    pub fn rerender(&mut self) -> Result<(), std::io::Error> {
        self.buffer.copy_from_slice(&self.back_buffer);
        let string_diffs = self.to_string();
        let changes = EscapeBuilder::new()
            .clear_screen()
            .move_to(0, 0)
            .write(string_diffs.into())
            .build();
        stdout().write_all(changes.as_bytes())?;
        stdout().flush()
    }

    pub fn produce_diffs(&self) -> String {
        let mut escape = EscapeBuilder::new();

        let mut prev_pos = None;

        for y in 0..self.height {
            let row_offs = y * self.width;
            for x in 0..self.width {
                let index = row_offs + x;
                let cell = self.back_buffer[index];
                if cell != self.buffer[index] {
                    if prev_pos != Some((x.saturating_sub(1), y)) {
                        escape = escape.move_to(x, y);
                    }
                    prev_pos = Some((x, y));
                    escape = escape.write(cell.character.to_string().into());
                }
            }
        }

        if self.cursor_pos != self.prev_cursor_pos {
            escape = escape.move_to(self.cursor_pos.0, self.cursor_pos.1);
        }

        escape.build()
    }

    pub fn put_char(&mut self, x: usize, y: usize, character: char) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = y * self.width + x;
        self.back_buffer[index].character = character;
    }

    fn to_string(&self) -> String {
        let mut result = "".to_string();
        for i in 0..self.height {
            for j in 0..self.width {
                let index = i * self.width + j;
                result.push(self.buffer[index].character);
            }
            result.push('\n');
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window() {
        let mut window = Window::new();
        window.resize(5, 5);
        for i in 0..10 {
            window.put_char(i, i, 'X');
        }

        std::mem::swap(&mut window.buffer, &mut window.back_buffer);

        assert_eq!(window.to_string(), "X    \n X   \n  X  \n   X \n    X\n");
    }

    #[test]
    fn test_diffs() {
        let mut window = Window::new();
        window.resize(5, 5);
        for i in 0..10 {
            window.put_char(i, 0, 'X');
        }

        let diffs = window.produce_diffs();
        assert_eq!(
            diffs,
            EscapeBuilder::new()
                .move_to(0, 0)
                .write("XXXXX".into())
                .build()
        );
    }
}
