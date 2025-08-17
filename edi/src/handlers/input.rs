use crate::{
    app::{handle_action, state::State},
    event::{self, manager, sender::EventBuffer, Event},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl manager::Handler<State> for Handler {
    fn handle(&mut self, app_state: &mut State, event: &Event, buf: &mut EventBuffer) {
        let Event::Input(input) = event else {
            return;
        };

        let _span = edi_lib::span!("input");

        let actions = app_state.mapper.map_input(input, app_state.mode);
        for action in actions {
            if let Err(err) = handle_action(action, app_state, buf) {
                edi_lib::debug!("{err}");
            }
        }
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty() == event::Type::Input
    }
}
