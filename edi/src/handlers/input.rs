use edi_lib::brand::Id;

use crate::{
    app::{handle_action, state::State, Mode},
    controller::{self, Handle},
    event::{self, Event, Payload},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl controller::EventHandler<State> for Handler {
    fn handle(&mut self, app_state: &mut State, event: &Event, ctrl: &mut Handle<State>) {
        let Payload::Input(input) = event.payload() else {
            return;
        };

        let _span = edi_lib::span!("input");

        let active_mode = app_state
            .buffers
            .active_buffer_mode()
            .unwrap_or(Mode::Normal);

        let actions = app_state.mapper.map_input(input, active_mode);
        for action in actions {
            if let Err(err) = handle_action(action, app_state, ctrl) {
                edi_lib::debug!("{err}");
            }
        }
    }

    fn interested_in(&self, _own_id: Id, event: &Event) -> bool {
        event.ty() == event::Type::Input
    }
}
