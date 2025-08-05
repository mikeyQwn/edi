use edi_frame::prelude::*;
use edi_frame::rect::Rect;

use crate::{
    app::AppState,
    event::{self, manager::Handler, sender::EventBuffer, Event},
};

pub struct DrawHandler;

impl DrawHandler {
    pub const fn new() -> Self {
        Self
    }
}

impl Handler<AppState> for DrawHandler {
    fn handle(&mut self, app_state: &mut AppState, _event: &Event, _buf: &mut EventBuffer) {
        let _span = edi_lib::span!("draw");

        let AppState { state, window } = app_state;

        edi_lib::debug!("drawing {} buffers", state.buffers.len());

        window.clear();

        state.buffers.iter_mut().rev().for_each(|(b, m)| {
            m.normalize(b);
            let mut bound = Rect::new_in_origin(m.size.x, m.size.y).bind(window);
            bound.clear();
            b.flush(&mut bound, &m.flush_options);
        });

        if let Err(err) = window.render() {
            edi_lib::debug!("{err}");
        }
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::Redraw
    }
}
