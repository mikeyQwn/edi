pub mod manager;
pub mod sender;
pub mod source;
pub mod sources;

use edi_term::input::Input;
use sender::Sender;

#[derive(Debug, Clone)]
pub struct Event {
    pub ty: Type,
    pub payload: Option<Payload>,
}

impl Event {
    #[must_use]
    pub const fn new(ty: Type) -> Self {
        Self { ty, payload: None }
    }

    #[must_use]
    pub fn input(input: Input) -> Self {
        Self::new(Type::Input).with_payload(Payload::Input(input))
    }

    #[must_use]
    pub fn write_char(c: char) -> Self {
        Self::new(Type::WriteChar).with_payload(Payload::WriteChar(c))
    }

    #[must_use]
    pub fn delete_char() -> Self {
        Self::new(Type::DeleteChar)
    }

    #[must_use]
    pub const fn redraw() -> Self {
        Self::new(Type::Redraw)
    }

    #[must_use]
    pub const fn quit() -> Self {
        Self::new(Type::Quit)
    }

    #[must_use]
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn is_quit(&self) -> bool {
        self.ty == Type::Quit
    }
}

#[derive(Debug, Clone)]
pub enum Payload {
    Input(Input),
    WriteChar(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Input,
    WriteChar,
    DeleteChar,
    Redraw,
    Quit,
}
