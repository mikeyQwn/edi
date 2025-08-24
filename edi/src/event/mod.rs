pub mod emitter;
pub mod sender;
pub mod source;
pub mod sources;

pub use sender::Sender;
pub use source::Source;

use edi_lib::brand::Id;
use edi_term::input::Input;

use crate::app::{self};

#[derive(Debug)]
pub struct Event {
    source: Option<Id>,
    payload: Payload,
}

impl Event {
    pub fn new(source: Option<Id>, payload: Payload) -> Self {
        Self { source, payload }
    }

    pub fn without_source(payload: Payload) -> Self {
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
    ModeSwitched {
        buffer_id: Id,
        target_mode: app::Mode,
    },
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
}

impl Payload {
    pub fn ty(&self) -> Type {
        match self {
            Self::Input(_) => Type::Input,
            Self::ModeSwitched { .. } => Type::ModeSwitched,
            Self::CharWritten { .. } => Type::CharWritten,
            Self::CharDeleted { .. } => Type::CharDeleted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Input,
    ModeSwitched,
    CharWritten,
    CharDeleted,
}

impl Type {
    pub fn is_oneof(self, s: &[Type]) -> bool {
        s.iter().any(|&ty| ty == self)
    }
}
