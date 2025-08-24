use std::collections::{HashMap, VecDeque};

use edi_lib::brand::Id;
use edi_term::input::Input;

use crate::{
    app::{self, buffers},
    event::{Event, Payload},
    query::{self, Query, Type},
};

use super::handler;

// A handle to controller which allows to send queries and emit events
pub struct Handle<State> {
    handler_id: Option<Id>,

    query_handlers: HashMap<Type, Box<dyn handler::QueryHandler<State>>>,

    collected_events: VecDeque<Event>,
    collected_queries: VecDeque<Query>,
}

impl<'a, State> Handle<State> {
    pub(super) fn new(
        query_handlers: HashMap<Type, Box<dyn handler::QueryHandler<State>>>,
    ) -> Self {
        Self {
            handler_id: None,

            query_handlers,

            collected_events: VecDeque::new(),
            collected_queries: VecDeque::new(),
        }
    }

    pub fn query(&mut self, state: &mut State, payload: query::Payload) {
        let query = Query::new(self.handler_id, payload);

        self.run_query(state, query);
    }

    pub(super) fn run_query(&mut self, state: &mut State, query: query::Query) {
        let ty = query.ty();

        // TODO: the performance here sucks and we don't need a hash map anyway
        let Some(mut handler) = self.query_handlers.remove(&ty) else {
            edi_lib::debug!("no query handler found for query: {query:?}");
            return;
        };

        handler.handle(state, query, self);

        self.query_handlers.insert(ty, handler);
    }

    pub(super) fn query_async(&mut self, payload: query::Payload) {
        let query = Query::new(self.handler_id, payload);
        self.collected_queries.push_back(query);
    }

    pub(super) fn with_handler_id(&mut self, id: Id) -> &mut Self {
        self.handler_id = Some(id);
        self
    }

    pub(super) fn pop_event(&mut self) -> Option<Event> {
        self.collected_events.pop_front()
    }

    pub(super) fn pop_query(&mut self) -> Option<Query> {
        self.collected_queries.pop_front()
    }

    pub fn add_event(&mut self, payload: Payload) {
        let event = Event::new(self.handler_id, payload);
        self.collected_events.push_back(event);
    }

    #[allow(unused)]
    pub fn add_input(&mut self, input: Input) {
        self.add_event(Payload::Input(input));
    }

    #[allow(unused)]
    pub fn add_switch_mode(&mut self, selector: buffers::Selector, target_mode: app::Mode) {
        self.add_event(Payload::SwitchMode {
            selector,
            target_mode,
        });
    }

    #[allow(unused)]
    pub fn add_write_char(&mut self, c: char) {
        self.add_event(Payload::WriteChar(c));
    }

    #[allow(unused)]
    pub fn add_delete_char(&mut self) {
        self.add_event(Payload::DeleteChar);
    }

    #[allow(unused)]
    pub fn add_char_written(&mut self, buffer_id: Id, offset: usize, c: char) {
        self.add_event(Payload::CharWritten {
            buffer_id,
            offset,
            c,
        });
    }

    #[allow(unused)]
    pub fn add_char_deleted(&mut self, buffer_id: Id, offset: usize, c: char) {
        self.add_event(Payload::CharDeleted {
            buffer_id,
            offset,
            c,
        });
    }

    pub fn add_undo(&mut self, selector: buffers::Selector) {
        self.add_event(Payload::Undo(selector));
    }

    pub fn add_redo(&mut self, selector: buffers::Selector) {
        self.add_event(Payload::Redo(selector));
    }

    pub fn query_redraw(&mut self) {
        self.query_async(query::Payload::Redraw);
    }

    pub fn query_quit(&mut self) {
        self.query_async(query::Payload::Quit);
    }
}
