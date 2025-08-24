pub mod emitter;
pub mod handler;
pub mod sender;
pub mod source;
pub mod sources;

pub use sender::Sender;
pub use source::Source;

use edi_lib::brand::Id;
use edi_term::input::Input;

use crate::app::{self, buffers};

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
    SwitchMode {
        selector: buffers::Selector,
        target_mode: app::Mode,
    },
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
    Undo(buffers::Selector),
    Redo(buffers::Selector),
}

impl Payload {
    pub fn ty(&self) -> Type {
        match self {
            Self::Input(_) => Type::Input,
            Self::SwitchMode { .. } => Type::SwtichMode,
            Self::WriteChar(_) => Type::WriteChar,
            Self::DeleteChar => Type::DeleteChar,
            Self::CharWritten { .. } => Type::CharWritten,
            Self::CharDeleted { .. } => Type::CharDeleted,
            Self::Undo(_) => Type::Undo,
            Self::Redo(_) => Type::Redo,
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
    Undo,
    Redo,
}

impl Type {
    pub fn is_oneof(self, s: &[Type]) -> bool {
        s.iter().any(|&ty| ty == self)
    }
}
