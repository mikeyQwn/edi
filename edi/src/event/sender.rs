use std::{collections::VecDeque, sync::mpsc};

use edi_lib::brand::Id;
use edi_term::input::Input;

use crate::app::{self, buffers};

use super::{Event, Payload};

#[derive(Debug)]
pub struct EventBuffer {
    handler_id: Option<Id>,
    events: VecDeque<Event>,
}

impl EventBuffer {
    #[must_use]
    pub(super) fn new() -> Self {
        Self {
            handler_id: None,
            events: VecDeque::default(),
        }
    }

    pub(super) fn with_id(&mut self, id: Id) -> &mut Self {
        self.handler_id = Some(id);
        self
    }

    #[must_use]
    pub(super) fn pop_first(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn add_event(&mut self, payload: Payload) {
        let event = Event::new(self.handler_id, payload);
        self.events.push_back(event);
    }

    #[allow(unused)]
    pub fn add_input(&mut self, input: Input) {
        self.add_event(Payload::Input(input));
    }

    #[allow(unused)]
    pub fn add_switch_mode(&mut self, selector: buffers::Selector, target_mode: app::Mode) {
        self.add_event(Payload::SwitchMode {
            selector,
            target_mode,
        });
    }

    #[allow(unused)]
    pub fn add_write_char(&mut self, c: char) {
        self.add_event(Payload::WriteChar(c));
    }

    #[allow(unused)]
    pub fn add_delete_char(&mut self) {
        self.add_event(Payload::DeleteChar);
    }

    #[allow(unused)]
    pub fn add_char_written(&mut self, buffer_id: Id, offset: usize, c: char) {
        self.add_event(Payload::CharWritten {
            buffer_id,
            offset,
            c,
        });
    }

    #[allow(unused)]
    pub fn add_char_deleted(&mut self, buffer_id: Id, offset: usize, c: char) {
        self.add_event(Payload::CharDeleted {
            buffer_id,
            offset,
            c,
        });
    }

    pub fn add_redraw(&mut self) {
        self.add_event(Payload::Redraw);
    }

    pub fn add_quit(&mut self) {
        self.add_event(Payload::Quit);
    }
}

pub struct Sender {
    pub(super) tx: mpsc::Sender<Payload>,
}

impl Sender {
    pub fn send_event(&self, event: Payload) -> bool {
        self.tx.send(event).is_ok()
    }

    pub fn send_input(&self, input: Input) -> bool {
        self.send_event(Payload::Input(input))
    }
}
