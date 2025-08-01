use edi_term::window::Window;

use crate::{
    app::{handle_action, State},
    event::{Event, Handler, Payload, Type},
};

pub struct InputHandler {
    state: State,
    window: Window,
}

impl InputHandler {
    pub fn new(state: State, window: Window) -> Self {
        Self { state, window }
    }
}

impl Handler for InputHandler {
    fn handle(&mut self, event: &super::Event, sender: &super::Sender) {
        let Some(Payload::Input(input)) = event.payload.as_ref() else {
            return;
        };

        let actions = self.state.mapper.map_input(input, self.state.mode);
        for action in actions {
            match handle_action(action, &mut self.state, &mut self.window) {
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
