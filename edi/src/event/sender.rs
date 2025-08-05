use std::{collections::VecDeque, sync::mpsc};

use edi_term::input::Input;

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
        self.add_event(Event::input(input));
    }

    #[allow(unused)]
    pub fn add_write_char(&mut self, c: char) {
        self.add_event(Event::write_char(c));
    }

    #[allow(unused)]
    pub fn add_delete_char(&mut self) {
        self.add_event(Event::delete_char());
    }

    pub fn add_redraw(&mut self) {
        self.add_event(Event::redraw());
    }

    pub fn add_quit(&mut self) {
        self.add_event(Event::quit());
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
        self.send_event(Event::input(input))
    }

    #[allow(unused)]
    pub fn send_redraw(&self) -> bool {
        self.send_event(Event::redraw())
    }

    #[allow(unused)]
    pub fn send_quit(&self) -> bool {
        self.send_event(Event::quit())
    }
}
