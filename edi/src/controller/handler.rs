use edi_lib::brand::Id;

use crate::{event::Event, query::Query};

use super::Handle;

pub trait EventHandler<State> {
    // TODO: make the state immutable
    fn handle(&mut self, state: &mut State, event: &Event, ctrl: &mut Handle<State>);
    fn interested_in(&self, own_id: Id, event: &Event) -> bool {
        let _ = (own_id, event);
        true
    }
}

pub trait QueryHandler<State> {
    fn handle(&mut self, state: &mut State, query: Query, ctrl: &mut Handle<State>);

    fn check_event(&mut self, state: &State, event: &Event, ctrl: &mut Handle<State>) {
        let _ = (state, event, ctrl);
    }

    fn interested_in(&self, own_id: Id, event: &Event) -> bool {
        let _ = (event, own_id);
        false
    }
}
