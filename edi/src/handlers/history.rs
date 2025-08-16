use std::{collections::HashMap, default};

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
    // Remove `length` characters starting from `offset`
    Delete { offset: usize, length: usize },
}

#[derive(Debug, Default)]
struct History {
    changes: Vec<Change>,
    current_position: usize,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_furute(&mut self, change: Change) {
        if self.current_position != self.changes.len() {
            self.changes.truncate(self.current_position);
        }

        self.changes.push(change);
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

    fn char_written(&mut self, event: &Event) {
        let Some(Payload::CharWritten {
            buffer_id,
            offset,
            c,
        }) = event.payload
        else {
            return;
        };

        let history = self
            .id_to_history
            .entry(buffer_id)
            .or_insert(History::default());

        history.write_furute(Change::Write {
            offset,
            content: String::from(c),
        });
    }

    fn char_deleted(&mut self, event: &Event) {
        let Some(Payload::CharDeleted { buffer_id, offset }) = event.payload else {
            return;
        };

        let history = self
            .id_to_history
            .entry(buffer_id)
            .or_insert(History::default());

        history.write_furute(Change::Delete { offset, length: 1 });
    }
}

impl manager::Handler<State> for Handler {
    fn handle(&mut self, _state: &mut State, event: &Event, _buf: &mut EventBuffer) {
        let _span = edi_lib::span!("history");

        match event.ty {
            event::Type::CharWritten => self.char_written(event),
            event::Type::CharDeleted => self.char_deleted(event),
            _ => {
                return;
            }
        }

        edi_lib::debug!("history changed, new history: {:?}", self.id_to_history);
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::CharWritten || event.ty == event::Type::CharDeleted
    }
}
