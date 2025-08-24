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
pub enum SpawnQuery {
    TerminalBuffer,
}

#[derive(Debug)]
pub enum MoveQuery {
    // TODO: Use query's own action
    Action {
        action: app::action::MoveAction,
        repeat: usize,
    },
}

#[derive(Debug)]
pub struct CommandQuery {
    pub command: String,
}

#[derive(Debug)]
pub enum Payload {
    Write(WriteQuery),
    History(HistoryQuery),
    Spawn(SpawnQuery),
    Move(MoveQuery),
    Command(CommandQuery),
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
            Self::Spawn(_) => Type::Spawn,
            Self::Move(_) => Type::Move,
            Self::Command(_) => Type::Command,
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
    Spawn,
    Move,
    Command,
    SwitchMode,
    Redraw,
    Quit,
}

impl Type {
    pub fn all() -> impl IntoIterator<Item = Type> {
        [
            Self::Write,
            Self::History,
            Self::Spawn,
            Self::Move,
            Self::Command,
            Self::SwitchMode,
            Self::Redraw,
            Self::Quit,
        ]
    }
}
