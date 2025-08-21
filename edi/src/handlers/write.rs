use edi::string::highlight::get_highlights;
use edi_lib::brand::Id;

use crate::{
    app::state::State,
    event::{self, manager, sender::EventBuffer, Event, Payload},
};

pub struct Handler;

impl Handler {
    pub const fn new() -> Self {
        Self
    }
}

impl manager::Handler<State> for Handler {
    fn handle(&mut self, app_state: &mut State, event: &Event, buf: &mut EventBuffer) {
        let _span = edi_lib::span!("write");

        match event.payload() {
            &Payload::WriteChar(c) => Self::write_char(app_state, c, buf),
            &Payload::DeleteChar => Self::delete_char(app_state, buf),
            _ => return,
        }

        buf.add_redraw();
    }

    fn interested_in(&self, _own_id: Id, event: &Event) -> bool {
        let types = &[event::Type::WriteChar, event::Type::DeleteChar];
        event.ty().is_oneof(types)
    }
}

impl Handler {
    fn write_char(state: &mut State, c: char, buf: &mut EventBuffer) {
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
            buf,
        );
    }

    fn delete_char(state: &mut State, buf: &mut EventBuffer) {
        state.within_active_buffer(
            |mut buffer, meta| {
                buffer.delete();
                meta.flush_options.highlights =
                    get_highlights(&buffer.as_ref().inner, &meta.filetype);
            },
            buf,
        );
    }
}
