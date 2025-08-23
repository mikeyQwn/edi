use crate::app::{self, buffers::Selector};

#[derive(Debug)]
pub enum Query {
    SwitchMode {
        buffer_selector: Selector,
        target: app::Mode,
    },
}

impl Query {
    pub fn ty(&self) -> QueryType {
        match self {
            Self::SwitchMode { .. } => QueryType::SwitchMode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum QueryType {
    SwitchMode,
}
