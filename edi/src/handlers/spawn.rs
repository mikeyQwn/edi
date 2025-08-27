use edi::buffer::Buffer;
use edi_frame::unit::Unit;
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
        let buffer_size = Vec2::new(Unit::full_width(), Unit::Cells(1));
        let buffer_offset = Vec2::new(Unit::zero(), Unit::half_height());

        let mut buffer = Buffer::new(":");
        buffer.cursor_offset = 1;

        state.buffers.attach_first(
            buffer,
            BufferMeta::new(Mode::Terminal)
                .with_size(buffer_size)
                .with_offset(buffer_offset)
                .with_statusline(false),
        );
    }
}
