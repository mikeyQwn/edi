use edi_lib::brand::Id;

use crate::app::{self, buffers::Selector};

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

    pub fn is_quit(&self) -> bool {
        self.ty() == Type::Quit
    }
}

#[derive(Debug)]
pub enum Payload {
    Quit,
}

impl Payload {
    pub fn ty(&self) -> Type {
        match self {
            Self::Quit { .. } => Type::Quit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Quit,
}
