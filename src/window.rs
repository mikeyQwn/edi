use std::io::{stdout, Write};

use crate::terminal::Terminal;

#[derive(Clone, Copy)]
pub struct Cell {
    character: char,
}

impl Default for Cell {
    fn default() -> Self {
        Self { character: ' ' }
    }
}

// Assumes that the terminal is in raw mode
pub struct Window {
    width: usize,
    height: usize,

    buffer: Vec<Cell>,
    back_buffer: Vec<Cell>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,

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
        // TODO: Add diff checking
        self.buffer.copy_from_slice(&self.back_buffer);

        stdout().write_all(self.to_string().as_bytes())?;
        stdout().flush()
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
}
