use crate::{
    app::{handle_action, AppState},
    event::{self, Event},
};

pub struct InputHandler {}

impl InputHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl event::Handler<AppState> for InputHandler {
    fn handle(&mut self, app_state: &mut AppState, event: &Event, sender: &event::Sender) {
        let Some(event::Payload::Input(input)) = event.payload.as_ref() else {
            return;
        };

        let _span = edi_lib::span!("input");

        let actions = app_state
            .state
            .mapper
            .map_input(input, app_state.state.mode);
        for action in actions {
            match handle_action(action, &mut app_state.state, sender) {
                Err(err) => {
                    edi_lib::debug!("{err}")
                }
                _ => {}
            }
        }
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::Input
    }
}
