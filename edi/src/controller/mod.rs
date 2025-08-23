pub mod handle;
pub mod handler;

pub use handle::Handle;

use std::{collections::HashMap, sync::mpsc};

use edi_lib::brand::Tag;

use crate::{
    event::{self, source::SourcesHandle, Event},
    query::{Query, QueryType},
};

pub struct Controller<State> {
    tag: Tag,

    query_tx: mpsc::Sender<Query>,
    query_rx: mpsc::Receiver<Query>,

    event_tx: mpsc::Sender<Event>,
    event_rx: mpsc::Receiver<Event>,

    event_sources: Vec<Box<dyn event::Source>>,
    event_handlers: Vec<Box<dyn handler::EventHandler<State>>>,

    query_handlers: HashMap<QueryType, Box<dyn handler::QueryHandler<State>>>,
}

impl<State> Controller<State> {
    pub fn new() -> Self {
        let tag = Tag::new();
        let (query_tx, query_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        Self {
            tag,

            query_tx,
            query_rx,

            event_tx,
            event_rx,

            event_sources: Vec::new(),
            event_handlers: Vec::new(),

            query_handlers: HashMap::new(),
        }
    }

    pub fn attach_source<Src>(&mut self, source: Src)
    where
        Src: event::Source + Send + 'static,
    {
        self.event_sources.push(Box::new(source));
    }

    pub fn run(mut self, state: State) {
        let mut sources_handle = SourcesHandle::new(self.event_sources.len());
        let sources = std::mem::take(&mut self.event_sources);
    }
}
