//! All methods that mutate buffer's inner string

use super::Buffer;

#[derive(Debug)]
pub enum Change {
    // Write `content` at `position`
    Write { position: usize, content: String },
    // Remove `length` characters starting from `position`
    Delete { position: usize, length: usize },
}

#[derive(Debug, Default, Clone, Copy)]
enum HistoryDirection {
    Past,
    #[default]
    Future,
}

#[derive(Debug, Default)]
pub struct ChangeHistory {
    past: Vec<Change>,
    future: Vec<Change>,
}

impl ChangeHistory {
    fn write_new(&mut self, change: Change, direction: HistoryDirection) {
        match direction {
            HistoryDirection::Future => {
                self.past.push(change);
            }
            HistoryDirection::Past => {
                self.future.push(change);
            }
        }
    }

    fn pop_undo(&mut self) -> Option<Change> {
        self.past.pop()
    }

    fn pop_redo(&mut self) -> Option<Change> {
        self.future.pop()
    }
}

impl Buffer {
    /// Writes a new character at cursor position
    pub fn write(&mut self, c: char) {
        self.history.future.clear();
        let undo_change = self.apply_write(self.cursor_offset, c);
        self.history.write_new(undo_change, Default::default());
    }

    /// Deletes a single character at cursor position
    pub fn delete(&mut self) {
        let Some(undo_change) = self.apply_delete(self.cursor_offset) else {
            return;
        };
        self.history.write_new(undo_change, Default::default());
    }

    /// Applies a single character write and emits a change that undoes it
    fn apply_write(&mut self, position: usize, c: char) -> Change {
        let undo_change = Change::Delete {
            position: position + 1,
            length: 1,
        };
        self.cursor_offset = position;

        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        self.inner.insert(self.cursor_offset, encoded);
        self.cursor_offset += 1;

        undo_change
    }

    /// Applies a single character delete, if not at the start of the rope, and emits a change that undoes it
    fn apply_delete(&mut self, position: usize) -> Option<Change> {
        self.cursor_offset = position.checked_sub(1)?;
        let deleted_char = self.inner.get(self.cursor_offset)?;
        let undo_change = Change::Write {
            position: self.cursor_offset,
            content: String::from(deleted_char),
        };
        self.inner.delete(self.cursor_offset..=self.cursor_offset);
        Some(undo_change)
    }

    pub fn undo(&mut self) {
        let Some(change) = self.history.pop_undo() else {
            return;
        };
        self.apply_change(change, HistoryDirection::Past);
    }

    pub fn redo(&mut self) {
        let Some(change) = self.history.pop_redo() else {
            return;
        };
        self.apply_change(change, HistoryDirection::Future);
    }

    fn apply_change(&mut self, change: Change, direction: HistoryDirection) {
        match change {
            Change::Write { position, content } => {
                content.chars().enumerate().for_each(|(offset, c)| {
                    let undo_change = self.apply_write(position + offset, c);
                    self.history.write_new(undo_change, direction);
                });
            }
            Change::Delete { position, length } => {
                (position..(position + length)).for_each(|position| {
                    let Some(undo_change) = self.apply_delete(position) else {
                        return;
                    };
                    self.history.write_new(undo_change, direction);
                })
            }
        }
    }
}
