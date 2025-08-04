use crate::{
    app::{handle_action, AppState},
    event::{self, manager::Handler, sender::EventBuffer, Event},
};

pub struct InputHandler;

impl InputHandler {
    pub const fn new() -> Self {
        Self
    }
}

impl Handler<AppState> for InputHandler {
    fn handle(&mut self, app_state: &mut AppState, event: &Event, buf: &mut EventBuffer) {
        let Some(event::Payload::Input(input)) = event.payload.as_ref() else {
            return;
        };

        let _span = edi_lib::span!("input");

        let actions = app_state
            .state
            .mapper
            .map_input(input, app_state.state.mode);
        for action in actions {
            if let Err(err) = handle_action(action, &mut app_state.state, buf) {
                edi_lib::debug!("{err}");
            }
        }
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::Input
    }
}
