use super::Payload;
use edi_term::input::Input;
use std::sync::mpsc;

pub struct Sender {
    tx: mpsc::Sender<Payload>,
}

impl Sender {
    pub const fn new(tx: mpsc::Sender<Payload>) -> Self {
        Self { tx }
    }

    pub fn send_event(&self, event: Payload) -> bool {
        self.tx.send(event).is_ok()
    }

    pub fn send_input(&self, input: Input) -> bool {
        self.send_event(Payload::Input(input))
    }
}
