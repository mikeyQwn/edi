use std::collections::{HashMap, VecDeque};

use edi_lib::brand::Id;
use edi_term::input::Input;

use crate::{
    app::{self, buffers::Selector},
    event::{Event, Payload},
    query::{
        self, CommandQuery, DrawQuery, HistoryQuery, MoveQuery, Query, SpawnQuery, Type, WriteQuery,
    },
};

use super::handler;

// A handle to controller which allows to send queries and emit events
pub struct Handle<State> {
    handler_id: Option<Id>,

    query_handlers: HashMap<Type, (Id, Box<dyn handler::QueryHandler<State>>)>,

    collected_events: VecDeque<Event>,
    collected_queries: VecDeque<Query>,
}

impl<'a, State> Handle<State> {
    pub(super) fn new(
        query_handlers: HashMap<Type, (Id, Box<dyn handler::QueryHandler<State>>)>,
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
        let prev_id = self.handler_id;
        let ty = query.ty();

        // TODO: the performance here sucks and we don't need a hash map anyway
        let Some((id, mut handler)) = self.query_handlers.remove(&ty) else {
            edi_lib::debug!("no query handler found for query: {query:?}");
            return;
        };

        handler.handle(state, query, self.with_handler_id(id));

        self.query_handlers.insert(ty, (id, handler));
        self.handler_id = prev_id;
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

    pub(super) fn check_event(&mut self, state: &mut State, event: &Event) {
        for ty in query::Type::all() {
            let Some((id, mut handler)) = self.query_handlers.remove(&ty) else {
                continue;
            };

            if handler.interested_in(id, event) {
                handler.check_event(state, event, self);
            }

            self.query_handlers.insert(ty, (id, handler));
        }
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

    pub fn query_history(&mut self, query: HistoryQuery) {
        self.query_async(query::Payload::History(query));
    }

    pub fn query_write(&mut self, query: WriteQuery) {
        self.query_async(query::Payload::Write(query));
    }

    pub fn query_switch_mode(&mut self, buffer_selector: Selector, target_mode: app::Mode) {
        self.query_async(query::Payload::SwitchMode {
            buffer_selector,
            target_mode,
        });
    }

    pub fn query_spawn(&mut self, query: SpawnQuery) {
        self.query_async(query::Payload::Spawn(query));
    }

    pub fn query_move(&mut self, query: MoveQuery) {
        self.query_async(query::Payload::Move(query));
    }

    pub fn query_command(&mut self, query: CommandQuery) {
        self.query_async(query::Payload::Command(query));
    }

    pub fn query_redraw(&mut self) {
        self.query_async(query::Payload::Draw(DrawQuery::Redraw));
    }

    pub fn query_draw(&mut self, query: DrawQuery) {
        self.query_async(query::Payload::Draw(query));
    }

    pub fn query_quit(&mut self) {
        self.query_async(query::Payload::Quit);
    }
}
