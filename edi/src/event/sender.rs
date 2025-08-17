use std::{collections::VecDeque, sync::mpsc};

use edi_lib::brand::Id;
use edi_term::input::Input;

use crate::app;

use super::Event;

#[derive(Debug)]
pub struct EventBuffer(VecDeque<Event>);

impl EventBuffer {
    #[must_use]
    pub(super) fn new() -> Self {
        Self(VecDeque::default())
    }

    #[must_use]
    pub(super) fn pop_first(&mut self) -> Option<Event> {
        self.0.pop_front()
    }

    pub fn add_event(&mut self, event: Event) {
        self.0.push_back(event);
    }

    #[allow(unused)]
    pub fn add_input(&mut self, input: Input) {
        self.add_event(Event::Input(input));
    }

    #[allow(unused)]
    pub fn add_switch_mode(&mut self, mode: app::Mode) {
        self.add_event(Event::SwitchMode(mode));
    }

    #[allow(unused)]
    pub fn add_write_char(&mut self, c: char) {
        self.add_event(Event::WriteChar(c));
    }

    #[allow(unused)]
    pub fn add_delete_char(&mut self) {
        self.add_event(Event::DeleteChar);
    }

    #[allow(unused)]
    pub fn add_char_written(&mut self, buffer_id: Id, offset: usize, c: char) {
        self.add_event(Event::CharWritten {
            buffer_id,
            offset,
            c,
        });
    }

    #[allow(unused)]
    pub fn add_char_deleted(&mut self, buffer_id: Id, offset: usize) {
        self.add_event(Event::CharDeleted { buffer_id, offset });
    }

    pub fn add_redraw(&mut self) {
        self.add_event(Event::Redraw);
    }

    pub fn add_quit(&mut self) {
        self.add_event(Event::Quit);
    }
}

pub struct Sender {
    pub(super) tx: mpsc::Sender<Event>,
}

impl Sender {
    pub fn send_event(&self, event: Event) -> bool {
        self.tx.send(event).is_ok()
    }

    pub fn send_input(&self, input: Input) -> bool {
        self.send_event(Event::Input(input))
    }
}
