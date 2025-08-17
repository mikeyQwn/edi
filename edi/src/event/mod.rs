pub mod emitter;
pub mod manager;
pub mod sender;
pub mod source;
pub mod sources;

use edi_lib::brand::Id;
use edi_term::input::Input;
use sender::Sender;

use crate::app;

#[derive(Debug, Clone)]
pub enum Event {
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
    },
    Redraw,
    Quit,
}

impl Event {
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
