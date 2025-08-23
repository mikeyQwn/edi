use edi_lib::brand::Id;

use crate::event::Event;

use super::Handle;

pub trait EventHandler<State> {
    // TODO: make the state immutable
    fn handle(&mut self, state: &mut State, event: &Event, ctrl: Handle<'_, State>);
    fn interested_in(&self, own_id: Id, event: &Event) -> bool {
        let _ = (own_id, event);
        true
    }
}

pub trait QueryHandler<State> {
    fn handle(&mut self, state: &mut State, event: &Event, ctrl: Handle<'_, State>);
    fn interested_in(&self, own_id: Id, event: &Event) -> bool {
        let _ = (own_id, event);
        true
    }
}
