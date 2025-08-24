use edi::string::highlight::get_highlights;

use crate::{
    app::state::State,
    controller::{self, Handle},
    query::{Payload, Query, WriteQuery},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl controller::QueryHandler<State> for Handler {
    fn handle(&mut self, app_state: &mut State, query: Query, ctrl: &mut Handle<State>) {
        let _span = edi_lib::span!("write");

        let Payload::Write(write_query) = query.payload() else {
            edi_lib::debug!(
                "non-write query submitted to write query handler, this is likely a bug"
            );
            return;
        };

        match *write_query {
            WriteQuery::WriteChar(c) => Self::write_char(app_state, c, ctrl),
            WriteQuery::DeleteChar => Self::delete_char(app_state, ctrl),
        }

        ctrl.query_redraw();
    }
}

impl Handler {
    fn write_char(state: &mut State, c: char, ctrl: &mut Handle<State>) {
        state.within_active_buffer(
            |mut buffer, meta| {
                let is_empty = buffer.as_ref().inner.is_empty();
                buffer.write(c);
                // Hack to always add a newline at the end of the file
                if is_empty {
                    buffer.write('\n');
                    buffer.set_cursor_offset(buffer.as_ref().cursor_offset - 1);
                }
                meta.flush_options.highlights =
                    get_highlights(&buffer.as_ref().inner, &meta.filetype);
            },
            ctrl,
        );
    }

    fn delete_char(state: &mut State, ctrl: &mut Handle<State>) {
        state.within_active_buffer(
            |mut buffer, meta| {
                buffer.delete();
                meta.flush_options.highlights =
                    get_highlights(&buffer.as_ref().inner, &meta.filetype);
            },
            ctrl,
        );
    }
}
