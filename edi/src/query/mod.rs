use edi_lib::brand::Id;

use crate::app::{
    self,
    buffers::{self, Selector},
};

#[derive(Debug)]
pub struct Query {
    #[allow(unused)]
    source: Option<Id>,
    payload: Payload,
}

impl Query {
    pub fn new(source: Option<Id>, payload: Payload) -> Self {
        Self { source, payload }
    }

    pub fn ty(&self) -> Type {
        self.payload.ty()
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn into_payload(self) -> Payload {
        self.payload
    }

    pub fn is_quit(&self) -> bool {
        self.ty() == Type::Quit
    }
}

#[derive(Debug)]
pub enum WriteQuery {
    WriteChar(char),
    DeleteChar,
}

#[derive(Debug)]
pub enum HistoryQuery {
    Undo(buffers::Selector),
    Redo(buffers::Selector),
}

#[derive(Debug)]
pub enum Payload {
    Write(WriteQuery),
    History(HistoryQuery),
    SwitchMode {
        buffer_selector: Selector,
        target_mode: app::Mode,
    },
    Redraw,
    Quit,
}

impl Payload {
    pub fn ty(&self) -> Type {
        match self {
            Self::Write(_) => Type::Write,
            Self::History(_) => Type::History,
            Self::SwitchMode { .. } => Type::SwitchMode,
            Self::Redraw => Type::Redraw,
            Self::Quit => Type::Quit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Write,
    History,
    SwitchMode,
    Redraw,
    Quit,
}

impl Type {
    pub fn all() -> impl IntoIterator<Item = Type> {
        [
            Self::Write,
            Self::History,
            Self::SwitchMode,
            Self::Redraw,
            Self::Quit,
        ]
    }
}
