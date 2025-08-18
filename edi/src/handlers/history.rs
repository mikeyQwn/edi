use std::collections::HashMap;

use edi_lib::brand::Id;

use crate::{
    app::state::State,
    event::{
        self,
        manager::{self},
        sender::EventBuffer,
        Event, Payload,
    },
};

#[derive(Debug)]
enum Change {
    // Write `content` at `offset`
    Write { offset: usize, content: String },
    // Remove `content` starting from `offset`
    Delete { offset: usize, content: String },
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
}

impl manager::Handler<State> for Handler {
    fn handle(&mut self, _state: &mut State, event: &Event, _buf: &mut EventBuffer) {
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
            &Payload::SwitchMode(_) => {
                // TODO: implement this
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
            event::Type::SwtichMode,
        ];
        event.ty().is_oneof(types)
    }
}
