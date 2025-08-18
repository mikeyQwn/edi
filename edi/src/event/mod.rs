pub mod emitter;
pub mod manager;
pub mod sender;
pub mod source;
pub mod sources;

use edi_lib::brand::Id;
use edi_term::input::Input;
use sender::Sender;

use crate::app;

#[derive(Debug)]
pub struct Event {
    source: Option<Id>,
    payload: Payload,
}

impl Event {
    pub(super) fn new(source: Option<Id>, payload: Payload) -> Self {
        Self { source, payload }
    }

    pub(super) fn without_source(payload: Payload) -> Self {
        Self::new(None, payload)
    }

    pub fn ty(&self) -> Type {
        self.payload().ty()
    }

    pub fn source_id(&self) -> Option<Id> {
        self.source
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

#[derive(Debug, Clone)]
pub enum Payload {
    Input(Input),
    SwitchMode(app::Mode),
    WriteChar(char),
    DeleteChar,
    CharWritten {
        buffer_id: Id,
        offset: usize,
        c: char,
    },
    CharDeleted {
        buffer_id: Id,
        offset: usize,
        c: char,
    },
    Redraw,
    Quit,
}

impl Payload {
    pub fn is_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }

    pub fn ty(&self) -> Type {
        match self {
            Self::Input(_) => Type::Input,
            Self::SwitchMode(_) => Type::SwtichMode,
            Self::WriteChar(_) => Type::WriteChar,
            Self::DeleteChar => Type::DeleteChar,
            Self::CharWritten { .. } => Type::CharWritten,
            Self::CharDeleted { .. } => Type::CharDeleted,
            Self::Redraw => Type::Redraw,
            Self::Quit => Type::Quit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Input,
    SwtichMode,
    WriteChar,
    DeleteChar,
    CharWritten,
    CharDeleted,
    Redraw,
    Quit,
}

impl Type {
    pub fn is_oneof(self, s: &[Type]) -> bool {
        s.iter().any(|&ty| ty == self)
    }
}
