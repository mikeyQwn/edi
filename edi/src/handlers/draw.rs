use edi_frame::prelude::*;
use edi_frame::rect::Rect;
use edi_lib::brand::Id;

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

        edi_lib::debug!(
            "drawing {buffer_count} buffers",
            buffer_count = state.buffers.len()
        );

        state.window.clear();
        state.buffers.iter_mut().rev().for_each(|bundle| {
            let (buffer, meta) = bundle.as_split_mut(buf);
            meta.normalize(buffer.as_ref());

            let mut bound = Rect::new_in_origin(meta.size.x, meta.size.y).bind(&mut state.window);
            bound.clear();
            buffer.as_ref().flush(&mut bound, &meta.flush_options);
        });

        if let Err(err) = state.window.render() {
            edi_lib::debug!("{err}");
        }
    }

    fn interested_in(&self, _own_id: Id, event: &Event) -> bool {
        event.ty() == event::Type::Redraw
    }
}
