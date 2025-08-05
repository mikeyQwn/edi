use edi::string::highlight::get_highlights;

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

        match event.ty {
            event::Type::WriteChar => Self::write_char(app_state, event),
            event::Type::DeleteChar => Self::delete_char(app_state),
            _ => {
                return;
            }
        }

        buf.add_redraw();
    }

    fn interested_in(&self, event: &Event) -> bool {
        event.ty == event::Type::WriteChar || event.ty == event::Type::DeleteChar
    }
}

impl Handler {
    fn write_char(state: &mut State, event: &Event) {
        let Some(Payload::WriteChar(c)) = event.payload else {
            return;
        };

        state.within_first_buffer(|buffer, meta| {
            let is_empty = buffer.inner.is_empty();
            buffer.write(c);
            // Hack to always add a newline at the end of the file
            if is_empty {
                buffer.write('\n');
                buffer.cursor_offset -= 1;
            }
            meta.flush_options.highlights = get_highlights(&buffer.inner, &meta.filetype);
        });
    }

    fn delete_char(state: &mut State) {
        state.within_first_buffer(|buffer, meta| {
            buffer.delete();
            meta.flush_options.highlights = get_highlights(&buffer.inner, &meta.filetype);
        });
    }
}
