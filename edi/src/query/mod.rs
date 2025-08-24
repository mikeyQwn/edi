use edi_lib::brand::Id;

use crate::app::buffers;

#[derive(Debug)]
pub struct Query {
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
    Redraw,
    Quit,
}

impl Payload {
    pub fn ty(&self) -> Type {
        match self {
            Self::Write(_) => Type::Write,
            Self::History(_) => Type::History,
            Self::Redraw => Type::Redraw,
            Self::Quit => Type::Quit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Write,
    History,
    Redraw,
    Quit,
}

impl Type {
    pub fn all() -> impl IntoIterator<Item = Type> {
        [Self::Write, Self::History, Self::Redraw, Self::Quit]
    }
}
