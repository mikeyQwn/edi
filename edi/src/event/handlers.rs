use crate::{
    app::{handle_action, AppState},
    event::{Event, Handler, Payload, Type},
};

pub struct InputHandler {}

impl InputHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Handler<AppState> for InputHandler {
    fn handle(&mut self, app_state: &mut AppState, event: &super::Event, sender: &super::Sender) {
        let Some(Payload::Input(input)) = event.payload.as_ref() else {
            return;
        };

        let actions = app_state
            .state
            .mapper
            .map_input(input, app_state.state.mode);
        for action in actions {
            match handle_action(action, &mut app_state.state, &mut app_state.window) {
                Ok(true) => {
                    let _ = sender.send_event(Event::new(Type::Quit));
                }
                Err(err) => {
                    edi_lib::debug!("{err}")
                }
                _ => {}
            }
        }
    }

    fn interested_in(&self, event: &super::Event) -> bool {
        event.ty == super::Type::Input
    }
}
