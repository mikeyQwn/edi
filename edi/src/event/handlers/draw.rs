use crate::{
    app::{redraw, AppState},
    event::{self, Event},
};

pub struct DrawHandler;

impl DrawHandler {
    pub const fn new() -> Self {
        Self
    }
}

impl event::Handler<AppState> for DrawHandler {
    fn handle(&mut self, app_state: &mut AppState, _event: &Event, _sender: &event::Sender) {
        let _span = edi_lib::span!("draw");

        if let Err(err) = redraw(&mut app_state.state, &mut app_state.window) {
            edi_lib::debug!("{err}");
        }
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::Redraw
    }
}
