pub mod handle;
pub mod handler;

pub use handle::Handle;
pub use handler::EventHandler;
pub use handler::QueryHandler;

use std::{collections::HashMap, sync::mpsc};

use edi_lib::brand::{Id, Tag};

use crate::{
    event::{self, source::SourcesHandle, Event},
    query::{Query, Type},
};

pub struct Controller<State> {
    tag: Tag,

    query_tx: mpsc::Sender<Query>,
    query_rx: mpsc::Receiver<Query>,

    event_tx: mpsc::Sender<event::Payload>,
    event_rx: mpsc::Receiver<event::Payload>,

    event_sources: Vec<Box<dyn event::Source>>,
    event_handlers: HashMap<Id, Box<dyn handler::EventHandler<State>>>,

    query_handlers: HashMap<Type, Box<dyn handler::QueryHandler<State>>>,

    // TODO: Make this piped queries
    piped_events: Vec<event::Payload>,
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
            event_handlers: HashMap::new(),

            query_handlers: HashMap::new(),

            piped_events: Vec::new(),
        }
    }

    pub fn pipe_event(&mut self, event: event::Payload) {
        self.piped_events.push(event);
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

    pub fn run(mut self, mut state: State) -> SourcesHandle {
        let mut sources_handle = SourcesHandle::new(self.event_sources.len());
        let sources = std::mem::take(&mut self.event_sources);
        let mut piped_events = std::mem::take(&mut self.piped_events);

        for mut source in sources {
            let sender = self.new_sender();
            sources_handle.add(std::thread::spawn(move || {
                source.run(sender);
            }));
        }

        let mut handle = Handle::new(std::mem::take(&mut self.query_handlers));

        while let Some(payload) = piped_events.pop() {
            Self::handle_event(
                self.event_handlers.iter_mut(),
                &Event::without_source(payload),
                &mut state,
                &mut handle,
            );
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
