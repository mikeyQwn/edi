pub mod handle;
pub mod handler;

pub use handle::Handle;
pub use handler::EventHandler;
pub use handler::QueryHandler;

use std::{collections::HashMap, sync::mpsc};

use edi_lib::brand::{Id, Tag};

use crate::query;
use crate::{
    event::{self, source::SourcesHandle, Event},
    query::Type,
};

pub struct Controller<State> {
    tag: Tag,

    event_tx: mpsc::Sender<event::Payload>,
    event_rx: mpsc::Receiver<event::Payload>,

    event_sources: Vec<Box<dyn event::Source>>,
    event_handlers: HashMap<Id, Box<dyn handler::EventHandler<State>>>,

    query_handlers: HashMap<Type, Box<dyn handler::QueryHandler<State>>>,

    piped_queries: Vec<query::Payload>,
}

impl<State> Controller<State> {
    pub fn new() -> Self {
        let tag = Tag::new();
        let (event_tx, event_rx) = mpsc::channel();

        Self {
            tag,

            event_tx,
            event_rx,

            event_sources: Vec::new(),
            event_handlers: HashMap::new(),

            query_handlers: HashMap::new(),

            piped_queries: Vec::new(),
        }
    }

    pub fn pipe_query(&mut self, query: query::Payload) {
        self.piped_queries.push(query);
    }

    pub fn attach_source<Src>(&mut self, source: Src)
    where
        Src: event::Source + Send + 'static,
    {
        self.event_sources.push(Box::new(source));
    }

    pub fn attach_event_handler<H>(&mut self, handler: H)
    where
        H: handler::EventHandler<State> + Send + 'static,
    {
        let id = self.tag.child_id();
        self.event_handlers.insert(id, Box::new(handler));
    }

    pub fn attach_query_handler<H>(&mut self, ty: query::Type, handler: H)
    where
        H: handler::QueryHandler<State> + Send + 'static,
    {
        self.query_handlers.insert(ty, Box::new(handler));
    }

    pub fn run(mut self, mut state: State) -> SourcesHandle {
        let mut sources_handle = SourcesHandle::new(self.event_sources.len());
        let sources = std::mem::take(&mut self.event_sources);
        let mut piped_queries = std::mem::take(&mut self.piped_queries);

        for mut source in sources {
            let sender = self.new_sender();
            sources_handle.add(std::thread::spawn(move || {
                source.run(sender);
            }));
        }

        let mut handle = Handle::new(std::mem::take(&mut self.query_handlers));

        while let Some(payload) = piped_queries.pop() {
            handle.query(&mut state, payload);
        }

        'outer: loop {
            if let Some(query) = handle.pop_query() {
                if query.is_quit() {
                    break 'outer;
                }

                handle.run_query(&mut state, query);
                continue 'outer;
            }

            if let Some(event) = handle.pop_event() {
                Self::handle_event(
                    self.event_handlers.iter_mut(),
                    &event,
                    &mut state,
                    &mut handle,
                );

                continue 'outer;
            }

            if let Ok(event) = self.event_rx.recv() {
                Self::handle_event(
                    self.event_handlers.iter_mut(),
                    &Event::without_source(event),
                    &mut state,
                    &mut handle,
                );
            }
        }

        sources_handle
    }

    fn new_sender(&mut self) -> event::Sender {
        event::Sender::new(mpsc::Sender::clone(&self.event_tx))
    }

    fn handle_event<'a>(
        handlers: impl IntoIterator<Item = (&'a Id, &'a mut Box<dyn handler::EventHandler<State>>)>,
        event: &'a Event,
        state: &'a mut State,
        ctrl: &mut Handle<State>,
    ) {
        for (&id, handler) in handlers {
            if !handler.interested_in(id, event) {
                continue;
            }

            handler.handle(state, event, ctrl.with_handler_id(id));
        }
    }
}
