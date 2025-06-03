//! All methods that mutate buffer's inner string

use super::Buffer;

#[derive(Debug)]
pub enum Change {
    Write { position: usize, content: String },
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
    pub fn write_new(&mut self, change: Change, direction: HistoryDirection) {
        match direction {
            HistoryDirection::Future => {
                self.past.push(change);
            }
            HistoryDirection::Past => {
                self.future.push(change);
            }
        }
    }

    pub fn pop_undo(&mut self) -> Option<Change> {
        self.past.pop()
    }

    pub fn pop_redo(&mut self) -> Option<Change> {
        self.future.pop()
    }
}

impl Buffer {
    pub fn write(&mut self, c: char) {
        self.history.future.clear();
        self.apply_write(self.cursor_offset, c, Default::default());
    }

    fn apply_write(&mut self, position: usize, c: char, direction: HistoryDirection) {
        self.cursor_offset = position;
        let is_empty = self.inner.is_empty();
        self.history.write_new(
            Change::Delete {
                position: self.cursor_offset + 1,
                length: 1,
            },
            direction,
        );
        self.inner
            .insert(self.cursor_offset, c.to_string().as_ref());

        if c == '\n' {
            self.cursor_offset += 1;
            return;
        }

        if is_empty {
            self.history.write_new(
                Change::Delete {
                    position: self.cursor_offset,
                    length: 1,
                },
                direction,
            );
            self.inner.insert(self.cursor_offset, "\n");
        }

        self.cursor_offset += 1;
    }

    pub fn delete(&mut self) {
        self.apply_delete(self.cursor_offset, Default::default());
    }

    fn apply_delete(&mut self, position: usize, direction: HistoryDirection) {
        self.cursor_offset = position;
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
            let Some(char) = self.inner.get(self.cursor_offset) else {
                return;
            };
            self.history.write_new(
                Change::Write {
                    position: self.cursor_offset,
                    content: String::from(char),
                },
                direction,
            );
            self.inner.delete(self.cursor_offset..=self.cursor_offset);
        }
    }

    pub fn undo(&mut self) {
        crate::debug!("undoing: {:?}", self.history);
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
                self.apply_write(position, content.chars().next().unwrap_or('0'), direction);
            }
            Change::Delete { position, .. } => {
                self.apply_delete(position, direction);
            }
        }
    }
}
