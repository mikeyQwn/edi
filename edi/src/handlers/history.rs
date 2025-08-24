use std::collections::HashMap;

use edi_lib::brand::Id;

use crate::{
    app::{buffer_bundle::BufferBundle, state::State},
    controller::{self, Handle},
    event::{self, emitter::buffer, Event, Payload},
    query::{self, HistoryQuery, Query},
};

#[derive(Debug)]
enum Change {
    // Write `content` at `offset`
    Write { offset: usize, content: String },
    // Remove `content` starting from `offset`
    Delete { offset: usize, content: String },
}

impl Change {
    fn undo(&self, buffer: &mut buffer::Buffer) {
        match self {
            Change::Write { offset, .. } => {
                buffer.set_cursor_offset(*offset + 1);
                buffer.delete();
            }

            Change::Delete { offset, content } => {
                buffer.set_cursor_offset(*offset - content.chars().count());
                for c in content.chars() {
                    buffer.write(c);
                }
            }
        }
    }

    fn apply(&self, buffer: &mut buffer::Buffer) {
        match self {
            Change::Delete { offset, .. } => {
                buffer.set_cursor_offset(*offset + 1);
                buffer.delete();
            }

            Change::Write { offset, content } => {
                buffer.set_cursor_offset(*offset);
                for c in content.chars() {
                    buffer.write(c);
                }
            }
        }
    }
}

#[derive(Debug)]
struct Record {
    age: usize,
    change: Change,
}

impl Record {
    pub fn new(age: usize, change: Change) -> Self {
        Self { age, change }
    }
}

#[derive(Debug, Default)]
struct History {
    changes: Vec<Record>,
    current_age: usize,
    current_position: usize,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_age(&mut self) {
        self.current_age = self.current_age.overflowing_add(1).0
    }

    fn new_record(&self, change: Change) -> Record {
        Record::new(self.current_age, change)
    }

    pub fn write_furute(&mut self, change: Change) {
        if self.current_position != self.changes.len() {
            self.changes.truncate(self.current_position);
        }

        self.changes.push(self.new_record(change));
        self.current_position = self.changes.len();
    }

    pub fn pop_record(&mut self) -> Option<&Record> {
        (self.current_position > 0).then_some(())?;
        self.current_position -= 1;
        self.changes.get(self.current_position)
    }

    pub fn return_record(&mut self) -> Option<&Record> {
        (self.current_position < self.changes.len()).then_some(())?;
        let item = self.changes.get(self.current_position);
        self.current_position += 1;
        item
    }
}

pub struct Handler {
    id_to_history: HashMap<Id, History>,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            id_to_history: HashMap::new(),
        }
    }

    fn char_written(&mut self, buffer_id: Id, offset: usize, c: char) {
        let history = self
            .id_to_history
            .entry(buffer_id)
            .or_insert(History::default());

        history.write_furute(Change::Write {
            offset,
            content: String::from(c),
        });
    }

    fn char_deleted(&mut self, buffer_id: Id, offset: usize, c: char) {
        let history = self
            .id_to_history
            .entry(buffer_id)
            .or_insert_with(History::new);

        history.write_furute(Change::Delete {
            offset,
            content: String::from(c),
        });
    }

    fn undo(&mut self, bundle: &mut BufferBundle, ctrl: &mut Handle<State>) {
        let Some(history) = self.id_to_history.get_mut(&bundle.id()) else {
            return;
        };

        let Some(record) = history.pop_record() else {
            return;
        };

        let mut buffer = bundle.buffer_mut(ctrl);

        record.change.undo(&mut buffer);
        let age = record.age;

        while let Some(record) = history.pop_record() {
            if record.age != age {
                history.return_record();
                return;
            }

            record.change.undo(&mut buffer);
        }
    }

    fn redo(&mut self, bundle: &mut BufferBundle, ctrl: &mut Handle<State>) {
        let Some(history) = self.id_to_history.get_mut(&bundle.id()) else {
            return;
        };

        let Some(record) = history.return_record() else {
            return;
        };

        let mut buffer = bundle.buffer_mut(ctrl);

        record.change.apply(&mut buffer);
        let age = record.age;

        while let Some(record) = history.return_record() {
            if record.age != age {
                history.pop_record();
                return;
            }

            record.change.apply(&mut buffer);
        }
    }
}

impl controller::QueryHandler<State> for Handler {
    fn handle(&mut self, state: &mut State, query: Query, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("history");

        let query::Payload::History(history_query) = query.payload() else {
            edi_lib::debug!(
                "non-history query submitted to history query handler, this is likely a bug"
            );
            return;
        };

        match history_query {
            HistoryQuery::Undo(selector) => {
                let Some(bundle) = state.buffers.get_mut(selector) else {
                    return;
                };
                self.undo(bundle, ctrl);
                ctrl.query_redraw();
            }
            HistoryQuery::Redo(selector) => {
                let Some(bundle) = state.buffers.get_mut(selector) else {
                    return;
                };
                self.redo(bundle, ctrl);
                ctrl.query_redraw();
            }
        }

        edi_lib::debug!("history changed, new history: {:?}", self.id_to_history);
    }

    fn check_event(&mut self, _state: &State, event: &Event, _ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("history");

        match event.payload() {
            &Payload::CharWritten {
                buffer_id,
                offset,
                c,
            } => self.char_written(buffer_id, offset, c),
            &Payload::CharDeleted {
                buffer_id,
                offset,
                c,
            } => self.char_deleted(buffer_id, offset, c),
            Payload::ModeSwitched { buffer_id, .. } => {
                self.id_to_history.get_mut(buffer_id).map(History::next_age);
                return;
            }
            _ => return,
        }

        edi_lib::debug!("history changed, new history: {:?}", self.id_to_history);
    }

    fn interested_in(&self, own_id: Id, event: &Event) -> bool {
        if event.source_id().is_some_and(|id| id.eq(&own_id)) {
            return false;
        }

        let types = &[
            event::Type::CharWritten,
            event::Type::CharDeleted,
            event::Type::ModeSwitched,
        ];
        event.ty().is_oneof(types)
    }
}
