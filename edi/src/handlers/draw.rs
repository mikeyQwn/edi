use edi_frame::prelude::*;
use edi_frame::rect::Rect;

use crate::{
    app::state::State,
    event::{
        self,
        manager::{self},
        sender::EventBuffer,
        Event,
    },
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl manager::Handler<State> for Handler {
    fn handle(&mut self, state: &mut State, _event: &Event, buf: &mut EventBuffer) {
        let _span = edi_lib::span!("draw");

        edi_lib::debug!("drawing {} buffers", state.buffers.len());

        state.window.clear();
        state.buffers.iter_mut().rev().for_each(|bundle| {
            let (mut b, m) = bundle.as_split_mut(buf);
            let b = b.as_mut();
            m.normalize(b);
            let mut bound = Rect::new_in_origin(m.size.x, m.size.y).bind(&mut state.window);
            bound.clear();
            b.flush(&mut bound, &m.flush_options);
        });

        if let Err(err) = state.window.render() {
            edi_lib::debug!("{err}");
        }
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::Redraw
    }
}
