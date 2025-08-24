use edi::buffer::Buffer;
use edi_lib::vec2::Vec2;

use crate::{
    app::{meta::BufferMeta, state::State, Mode},
    controller::{self, Handle},
    query::{Payload, Query, SpawnQuery},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl controller::QueryHandler<State> for Handler {
    fn handle(&mut self, state: &mut State, query: Query, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("write");

        let Payload::Spawn(spawn_query) = query.payload() else {
            edi_lib::debug!(
                "non-spawn query submitted to spawn query handler, this is likely a bug"
            );
            return;
        };

        match spawn_query {
            &SpawnQuery::TerminalBuffer => Self::spawn_terminal_buffer(state),
        }

        ctrl.query_redraw();
    }
}

impl Handler {
    fn spawn_terminal_buffer(state: &mut State) {
        let mut size = edi_term::get_size()
            .map(Vec2::from_dims)
            .unwrap_or(Vec2::new(10, 1))
            .map(|v| v as usize);
        size.y = 1;

        let mut buffer = Buffer::new(":");
        buffer.cursor_offset = 1;

        state
            .buffers
            .attach_first(buffer, BufferMeta::new(Mode::Terminal).with_size(size));
    }
}
