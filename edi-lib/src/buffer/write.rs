//! All methods that mutate buffer's inner string

use super::Buffer;

impl Buffer {
    /// Writes a new character at cursor position
    pub fn write(&mut self, c: char) {
        self.apply_write(self.cursor_offset, c);
    }

    /// Deletes a single character at cursor position
    pub fn delete(&mut self) -> Option<char> {
        self.apply_delete(self.cursor_offset)
    }

    fn apply_write(&mut self, position: usize, c: char) {
        self.cursor_offset = position;
        self.inner
            .insert(self.cursor_offset, c.encode_utf8(&mut [0_u8; 4]));
        self.cursor_offset += 1;
    }

    fn apply_delete(&mut self, position: usize) -> Option<char> {
        self.cursor_offset = position.checked_sub(1)?;
        let deleted_char = self.inner.get(self.cursor_offset)?;
        self.inner.delete(self.cursor_offset..=self.cursor_offset);

        Some(deleted_char)
    }
}
